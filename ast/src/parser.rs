use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;

use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;

use crate::errors::ParseError;
use crate::latex_constants::*;
use crate::node::{ExpandableData, LeafData, MathKind, Node, NodeRef, NodeRefWeak};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::{TexlaUuidProvider, Uuid};

type NodeParser<'a> = BoxedParser<'a, char, NodeRef, Simple<char>>;
type NodesParser<'a> = BoxedParser<'a, char, Vec<NodeRef>, Simple<char>>;

#[derive(Clone)]
struct LatexParser {
    uuid_provider: RefCell<TexlaUuidProvider>,
    portal: RefCell<HashMap<Uuid, NodeRefWeak>>,
}

pub(crate) fn parse_latex(string: String) -> Result<TexlaAst, ParseError> {
    // TODO: for performance, the parser should not be created every time, but reused
    // (could be realized by using reference arguments instead of attributes)
    let parser = LatexParser::new();
    let root = parser.parser().parse(string.clone())?;
    let highest_level = parser.highest_level(&string);
    Ok(TexlaAst {
        portal: parser.portal.into_inner(),
        uuid_provider: parser.uuid_provider.into_inner(),
        root,
        highest_level,
    })
}

const PARENTHESIS: (&'static str, &'static str) = ("(", ")");
const SQUARE_BRACKETS: (&'static str, &'static str) = ("[", "]");
const CURLY_BRACES: (&'static str, &'static str) = ("{", "}");

const METADATA_MARKER: &'static str = "% TEXLA METADATA";

impl LatexParser {
    // TODO Indentation Support
    fn new() -> Self {
        LatexParser {
            uuid_provider: RefCell::new(TexlaUuidProvider::new()),
            portal: RefCell::new(HashMap::new()),
        }
    }

    fn build_text(&self, text: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Text { text: text.clone() },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            text,
            metadata,
        )
    }

    fn build_comment(&self, comment: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Comment {
                comment: format!("% {}", comment.clone()),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("% {}", comment.clone()),
            metadata,
        )
    }

    fn build_math(&self, text: String, kind: MathKind) -> NodeRef {
        Node::new_leaf(
            LeafData::Math {
                kind: kind.clone(),
                content: text.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            match kind {
                MathKind::SquareBrackets => {
                    format!("\\[{}\\]", text.clone())
                }
                MathKind::DoubleDollars => {
                    format!("$${}$$", text.clone())
                }
                MathKind::Displaymath => {
                    format!("\\begin{{displaymath}}{}\\end{{displaymath}}", text.clone())
                }
                MathKind::Equation => {
                    format!("\\begin{{equation}}{}\\end{{equation}}", text.clone())
                }
            },
            Default::default(),
        )
    }

    fn build_image(
        &self,
        options: Option<String>,
        path: String,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_leaf(
            LeafData::Image {
                path: path.clone(),
                options: options.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            match options {
                None => {
                    format!("\\includegraphics{{{path}}}")
                }
                Some(option) => {
                    format!("\\includegraphics[{option}]{{{path}}}")
                }
            },
            metadata,
        )
    }

    fn build_caption(&self, caption: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Caption {
                caption: caption.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\caption{{{caption}}}"),
            metadata,
        )
    }

    fn build_label(&self, label: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Label {
                label: label.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\label{{{label}}}"),
            metadata,
        )
    }

    fn build_file(
        &self,
        path: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::File { path: path.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!(
                "% TEXLA FILE BEGIN {{{}}}\n...\n% TEXLA FILE END {{{}}}",
                &path, &path
            ),
            metadata,
        )
    }

    fn build_env(
        &self,
        name: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Environment { name: name.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\begin{{{}}}\n...\n\\end{{{}}}", name.clone(), name),
            metadata,
        )
    }

    fn build_segment(
        &self,
        heading: String,
        children: Vec<NodeRef>,
        raw: String,
        counted: bool,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading, counted },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("{raw}\n..."),
            metadata,
        )
    }

    fn build_document(
        &self,
        preamble: String,
        postamble: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Document {
                preamble,
                postamble,
            },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            String::new(),
            metadata,
        )
    }

    fn argument_surrounded_by(
        (start, end): (&'static str, &'static str),
    ) -> BoxedParser<char, String, Simple<char>> {
        none_of(end)
            .repeated()
            .at_least(1)
            .delimited_by(just(start), just(end))
            .collect::<String>()
            .boxed()
    }

    fn parser(&self) -> impl Parser<char, NodeRef, Error = Simple<char>> + '_ {
        let key_value_pair = text::ident()
            .then_ignore(just(":"))
            // TODO: this prevents leading or trailing spaces in values. Do we want that?
            .then(text::ident().padded())
            .map(|(key, value)| (key, value))
            .boxed();

        let metadata = just("% TEXLA METADATA")
            .padded()
            .ignore_then(
                key_value_pair
                    .separated_by(just(","))
                    .allow_trailing()
                    .delimited_by(just("("), just(")")),
            )
            .padded()
            .or_not()
            .map(|option| match option {
                Some(pairs) => pairs.into_iter().collect::<HashMap<String, String>>(),
                None => HashMap::new(),
            })
            .boxed();

        let comment = metadata
            .clone()
            .then_ignore(just("%").padded())
            .then(take_until(choice((
                newline().then(none_of("%").rewind()).to("END"),
                just("% TEXLA").rewind(),
            ))))
            .try_map(|(metadata, (comment, _)), span| {
                let string: String = comment.iter().collect();
                if string.starts_with("TEXLA") {
                    Err(Simple::custom(
                        span,
                        "found TEXLA Metadata instead of regular Comment",
                    ))
                } else {
                    Ok(self.build_comment(comment.iter().collect(), metadata))
                }
            })
            .padded()
            .boxed();

        let curly_braces = Self::argument_surrounded_by(CURLY_BRACES);

        let options = just("[")
            .ignore_then(none_of("]").repeated())
            .then_ignore(just("]"))
            .collect()
            .boxed();

        let image = metadata
            .clone()
            .then_ignore(just("\\includegraphics"))
            .then(options.or_not())
            .then(
                none_of("}")
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .delimited_by(just("{"), just("}")),
            )
            .padded()
            .map(|((metadata, options), path)| {
                self.build_image(options, path.parse().unwrap(), metadata)
            })
            .boxed();

        let math_double_dollars = take_until(just("$$").rewind())
            .delimited_by(just("$$"), just("$$"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::DoubleDollars))
            .boxed();

        let math_square_brackets = take_until(just("\\]").rewind())
            .delimited_by(just("\\["), just("\\]"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::SquareBrackets))
            .boxed();

        let math_equation = take_until(just("\\end{equation}").rewind())
            .delimited_by(just("\\begin{equation}"), just("\\end{equation}"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::Equation))
            .boxed();

        let math_displaymath = take_until(just("\\end{displaymath}").rewind())
            .delimited_by(just("\\begin{displaymath}"), just("\\end{displaymath}"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::Displaymath))
            .boxed();

        let math = choice((
            math_double_dollars,
            math_square_brackets,
            math_equation,
            math_displaymath,
        ))
        .padded()
        .boxed();

        let caption = metadata
            .clone()
            .then_ignore(just("\\caption"))
            .then(curly_braces.clone())
            .map(|(metadata, text)| self.build_caption(text, metadata))
            .padded()
            .boxed();

        let label = metadata
            .clone()
            .then_ignore(just("\\label"))
            .then(curly_braces.clone())
            .map(|(metadata, text)| self.build_label(text, metadata))
            .padded()
            .boxed();

        let terminator = choice((
            Self::segment_command_parser().rewind().to("segment"),
            just("\\begin").rewind(),
            just("\\end").rewind(),
            just("\\end{document}").rewind(), // TODO remove this line?
            just("%").rewind(),
            image.clone().to("image").rewind(),
            math.clone().to("math").rewind(),
            caption.clone().to("caption").rewind(),
            label.clone().to("label").rewind(),
            newline().then(newline()).to("\n\n"),
            // TODO recognize and consume also more than 2 newlines
        ))
        .boxed();

        let text_node = metadata
            .clone()
            .then(take_until(terminator))
            .try_map(|(metadata, (v, _)), span| {
                if !v.is_empty() {
                    Ok((metadata, v))
                } else {
                    Err(Simple::custom(span, "Found empty text".to_string()))
                }
            })
            .then_ignore(newline().or_not())
            .map(|(metadata, x)| {
                self.build_text(
                    x.iter().collect::<String>().trim_end().to_string(),
                    metadata,
                )
            })
            .boxed();

        let leaf = choice((
            image.clone(),
            math.clone(),
            caption.clone(),
            label.clone(),
            comment.clone(),
            text_node.clone(),
        ))
        .boxed();

        let environment = recursive(|environment| {
            metadata
                .clone()
                .then_ignore(just("\\begin"))
                .then(curly_braces.clone())
                .padded()
                .then(leaf.clone().or(environment).repeated())
                .then(just("\\end").ignore_then(curly_braces.clone()).padded())
                .try_map(|(((metadata, name_begin), children), name_end), span| {
                    if name_begin != name_end {
                        Err(Simple::custom(span, "Environment not closed correctly"))
                    } else {
                        Ok(self.build_env(name_end, children, metadata))
                    }
                })
        })
        .boxed();

        let prelude = choice((leaf.clone(), environment.clone())).boxed();

        let prelude_in_inputs = self.one_or_in_inputs(prelude.clone(), prelude);
        let preludes_in_inputs = prelude_in_inputs.clone().repeated();

        let subparagraph = self.segment(
            "subparagraph",
            prelude_in_inputs.clone(),
            prelude_in_inputs.clone(),
        );
        let subparagraphs_in_inputs =
            self.one_or_in_inputs(subparagraph, prelude_in_inputs.clone());

        let mut segment_in_inputs_parsers = vec![subparagraphs_in_inputs];

        LEVELS
            .iter()
            .rev()
            .enumerate()
            .skip(1) // subparagraph is already there
            .for_each(|(i, (_, keyword))| {
                let segment = self.segment(
                    keyword,
                    segment_in_inputs_parsers[i - 1].clone(),
                    prelude_in_inputs.clone(),
                );
                segment_in_inputs_parsers
                    .push(self.one_or_in_inputs(segment, prelude_in_inputs.clone()));
            });

        let root_children = preludes_in_inputs
            .clone()
            .then(
                choice(
                    segment_in_inputs_parsers
                        .into_iter()
                        .map(|parsers| parsers.repeated().at_least(1).boxed())
                        .collect::<Vec<NodesParser>>(),
                )
                .or(empty().padded().to(vec![])) // allow for empty documents
                .boxed(),
            )
            .boxed();

        let preamble = take_until(just("\\begin{document}").rewind())
            .map(|(preamble, _)| preamble.iter().collect())
            .boxed();

        let document = preamble
            .clone()
            .or_not()
            .then(metadata.clone())
            .then_ignore(just::<_, _, Simple<char>>("\\begin{document}").padded())
            .then(root_children.clone())
            .then_ignore(just("\\end{document}"))
            .map(|((preamble, metadata), (mut leaves, mut segments))| {
                self.build_document(
                    preamble.unwrap_or(String::new()),
                    String::new(),
                    {
                        leaves.append(&mut segments);
                        leaves
                    },
                    metadata,
                )
            })
            .then_ignore(end().padded())
            .boxed();
        document
    }

    fn metadata() -> impl Parser<char, HashMap<String, String>, Error = Simple<char>> + 'static {
        let key_value_pair = text::ident()
            .then_ignore(just(":"))
            // TODO: this prevents leading or trailing spaces in values. Do we want that?
            .then(text::ident().padded())
            .map(|(key, value)| (key, value))
            .boxed();

        just(METADATA_MARKER)
            .padded()
            .ignore_then(
                key_value_pair
                    .separated_by(just(","))
                    .allow_trailing()
                    .delimited_by(just("("), just(")")),
            )
            .padded()
            .or_not()
            .map(|option| match option {
                Some(pairs) => pairs.into_iter().collect::<HashMap<String, String>>(),
                None => HashMap::new(),
            })
            .boxed()
    }

    fn segment<'a>(
        &'a self,
        keyword: &'static str,
        next_level: NodeParser<'a>,
        prelude: NodeParser<'a>,
    ) -> BoxedParser<'a, char, NodeRef, Simple<char>> {
        // TODO: prevent \sectioning etc. from parsing (using keyword?)
        Self::metadata()
            .then_ignore(just("\\").then(just(keyword)))
            .then(just(UNCOUNTED_SEGMENT_MARKER).or_not())
            .then(Self::argument_surrounded_by(CURLY_BRACES))
            .then_ignore(newline().or_not())
            .then(prelude.repeated().padded())
            .then(next_level.repeated())
            .map(
                move |((((metadata, star), heading), mut blocks), mut subsegments)| {
                    blocks.append(&mut subsegments);
                    self.build_segment(
                        heading.clone(),
                        blocks,
                        format!("\\{keyword}{{{heading}}}"), // TODO: newline?
                        star.is_none(),
                        metadata,
                    )
                },
            )
            .boxed()
    }

    // this returns what was formerly named thing_any
    /// Accepts one thing or zero or more things in arbitrarily nested inputs.
    /// In all levels there can be zero or more preludes before things.
    fn one_or_in_inputs<'a>(
        &'a self,
        thing: impl Parser<char, NodeRef, Error = Simple<char>> + Clone + 'a,
        prelude: impl Parser<char, NodeRef, Error = Simple<char>> + 'a,
    ) -> BoxedParser<'a, char, NodeRef, Simple<char>> {
        recursive(|things_in_inputs| {
            Self::metadata()
                .then_ignore(just("% TEXLA FILE BEGIN"))
                .then(Self::argument_surrounded_by(CURLY_BRACES).padded())
                .then(prelude.repeated())
                .then(things_in_inputs.or(thing.clone()).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .then(Self::argument_surrounded_by(CURLY_BRACES).padded())
                .try_map(
                    |((((metadata, path), mut prelude), mut children), path_end), span| {
                        if path == path_end {
                            Ok(self.build_file(
                                path,
                                {
                                    prelude.append(&mut children);
                                    prelude
                                },
                                metadata,
                            ))
                        } else {
                            Err(Simple::custom(
                                span,
                                format!(
                                    "File opened: {path} but not closing tag was for {path_end}"
                                ),
                            ))
                        }
                    },
                )
        })
        .or(thing)
        .boxed()
    }

    fn highest_level(&self, string: &str) -> i8 {
        take_until(Self::segment_command_parser())
            .map(|(_, level)| level)
            .parse(string)
            .unwrap_or(LEAF_LEVEL)
    }

    fn segment_command_parser() -> impl Parser<char, i8, Error = Simple<char>> + 'static {
        // TODO find way to ignore \sectioning (use keyword?)
        choice(LEVELS.map(|(level, keyword)| {
            just::<char, &str, Simple<char>>("\\")
                .ignore_then(just(keyword))
                .to(level)
        }))
        .boxed()
    }
}
