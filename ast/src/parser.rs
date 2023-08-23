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
use crate::texla_constants::*;
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

impl LatexParser {
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
                comment: format!("{COMMENT_PREFIX} {comment}"),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("{COMMENT_PREFIX} {comment}"),
            metadata,
        )
    }

    fn build_math(&self, text: String, kind: &MathKind) -> NodeRef {
        Node::new_leaf(
            LeafData::Math {
                kind: kind.clone(),
                content: text.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            // this is not completely redundant to [LeafData::to_latex], because we can
            // normalize the input before displaying it here.
            match kind {
                MathKind::DoubleDollars => {
                    format!("{DOUBLE_DOLLARS}{text}{DOUBLE_DOLLARS}")
                }
                MathKind::SquareBrackets => {
                    format!("{SQUARE_BRACKETS_LEFT}{text}{SQUARE_BRACKETS_RIGHT}")
                }
                MathKind::Displaymath => {
                    format!("{DISPLAYMATH_BEGIN}{text}{DISPLAYMATH_END}")
                }
                MathKind::Equation => {
                    format!("{EQUATION_BEGIN}{text}{EQUATION_END}")
                }
                MathKind::Align => {
                    format!("{ALIGN_BEGIN}{text}{ALIGN_END}\n")
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
                    format!("{INCLUDEGRAPHICS}{{{path}}}")
                }
                Some(options_str) => {
                    format!("{INCLUDEGRAPHICS}{OPTIONS_BEGIN}{options_str}{OPTIONS_END}{{{path}}}")
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
            format!("{CAPTION}{{{caption}}}"),
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
            format!("{LABEL}{{{label}}}"),
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
                "{FILE_BEGIN_MARK}{{{path}}}\n{SKIPPED_CONTENT_MARK}\n{FILE_END_MARK}{{{path}}}"
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
            format!("{BEGIN}{{{name}}}\n{SKIPPED_CONTENT_MARK}\n{END}{{{name}}}"),
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
            format!("{raw}\n{SKIPPED_CONTENT_MARK}"),
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
        let metadata = Self::metadata();

        let comment = metadata
            .clone()
            .then_ignore(just(COMMENT_PREFIX).padded())
            .then(take_until(choice((
                newline().then(none_of(COMMENT_PREFIX).rewind()).to(""),
                just(TEXLA_COMMENT_PREFIX).rewind(),
            ))))
            .try_map(|(metadata, (comment, _)), span| {
                let string: String = comment.iter().collect();
                if string.starts_with(TEXLA) {
                    Err(Simple::custom(
                        span,
                        "found TeXLa metadata instead of regular comment",
                    ))
                } else {
                    Ok(self.build_comment(string, metadata))
                }
            })
            .padded()
            .boxed();

        let curly_brackets = Self::argument_surrounded_by(BLOCK_DELIMITERS);

        let options = just(OPTIONS_BEGIN)
            .ignore_then(none_of(OPTIONS_END).repeated())
            .then_ignore(just(OPTIONS_END))
            .collect()
            .boxed();

        let image = metadata
            .clone()
            .then_ignore(just(INCLUDEGRAPHICS))
            .then(options.or_not())
            .then(
                none_of(BLOCK_END)
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .delimited_by(just(BLOCK_BEGIN), just(BLOCK_END)),
            )
            .padded()
            .map(|((metadata, options), path)| {
                self.build_image(options, path.parse().unwrap(), metadata)
            })
            .boxed();

        let math_double_dollars =
            self.math_delimited_by(DOUBLE_DOLLARS, DOUBLE_DOLLARS, MathKind::DoubleDollars);
        let math_square_brackets = self.math_delimited_by(
            SQUARE_BRACKETS_LEFT,
            SQUARE_BRACKETS_RIGHT,
            MathKind::SquareBrackets,
        );
        let math_displaymath =
            self.math_delimited_by(DISPLAYMATH_BEGIN, DISPLAYMATH_END, MathKind::Displaymath);
        let math_equation =
            self.math_delimited_by(EQUATION_BEGIN, EQUATION_END, MathKind::Equation);
        let math_align = self.math_delimited_by(ALIGN_BEGIN, ALIGN_END, MathKind::Align);

        let math = choice((
            math_double_dollars,
            math_square_brackets,
            math_displaymath,
            math_equation,
            math_align,
        ))
        .padded()
        .boxed();

        let caption = metadata
            .clone()
            .then_ignore(just(CAPTION))
            .then(curly_brackets.clone())
            .map(|(metadata, text)| self.build_caption(text, metadata))
            .padded()
            .boxed();

        let label = metadata
            .clone()
            .then_ignore(just(LABEL))
            .then(curly_brackets.clone())
            .map(|(metadata, text)| self.build_label(text, metadata))
            .padded()
            .boxed();

        let terminator = choice((
            Self::segment_command_parser().rewind().to(""),
            just(BEGIN).rewind(),
            just(END).rewind(),
            just(COMMENT_PREFIX).rewind(),
            image.clone().rewind().to(""),
            math.clone().rewind().to(""),
            caption.clone().rewind().to(""),
            label.clone().rewind().to(""),
            newline().repeated().at_least(2).to(""),
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
                .then_ignore(just(BEGIN))
                .then(curly_brackets.clone())
                .padded()
                .then(leaf.clone().or(environment).repeated())
                .then(just(END).ignore_then(curly_brackets.clone()).padded())
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
            SUBPARAGRAPH,
            prelude_in_inputs.clone(),
            prelude_in_inputs.clone(),
        );
        let subparagraphs_in_inputs =
            self.one_or_in_inputs(subparagraph, prelude_in_inputs.clone());

        let mut segment_in_inputs_parsers = vec![subparagraphs_in_inputs];

        SEGMENT_LEVELS
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

        let preamble = take_until(just(DOCUMENT_BEGIN).rewind())
            .map(|(preamble, _)| preamble.iter().collect())
            .boxed();

        // document parser
        preamble
            .clone()
            .or_not()
            .then(metadata.clone())
            .then_ignore(just::<_, _, Simple<char>>(DOCUMENT_BEGIN).padded())
            .then(root_children.clone())
            .then_ignore(just(DOCUMENT_END))
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
            .boxed()
    }

    fn math_delimited_by<'a>(
        &'a self,
        begin: &'a str,
        end: &'a str,
        kind: MathKind,
    ) -> BoxedParser<'a, char, NodeRef, Simple<char>> {
        take_until(just(end).rewind())
            .delimited_by(just(begin), just(end))
            .map(move |(inner, _)| self.build_math(inner.iter().collect(), &kind))
            .boxed()
    }

    fn metadata() -> BoxedParser<'static, char, HashMap<String, String>, Simple<char>> {
        let key_value_pair = text::ident()
            .then_ignore(just(METADATA_SEPARATOR_KEY_VALUE))
            .then(
                none_of(vec![METADATA_DELIMITER_RIGHT, METADATA_SEPARATOR_VALUES])
                    .repeated()
                    .collect::<String>(),
            )
            .map(|(key, value)| (key, value))
            .boxed();

        just(METADATA_MARK)
            .padded()
            .ignore_then(
                key_value_pair
                    .separated_by(just(METADATA_SEPARATOR_VALUES))
                    .allow_trailing()
                    .delimited_by(
                        just(METADATA_DELIMITER_LEFT),
                        just(METADATA_DELIMITER_RIGHT),
                    ),
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
        Self::metadata()
            .then_ignore(just(KEYWORD_PREFIX).then(text::keyword(keyword)))
            .then(just(UNCOUNTED_SEGMENT_MARKER).or_not())
            .then(Self::argument_surrounded_by(BLOCK_DELIMITERS))
            .then_ignore(newline().or_not())
            .then(prelude.repeated().padded())
            .then(next_level.repeated())
            .map(
                move |((((metadata, star), heading), mut blocks), mut subsegments)| {
                    blocks.append(&mut subsegments);
                    self.build_segment(
                        heading.clone(),
                        blocks,
                        format!("{KEYWORD_PREFIX}{keyword}{{{heading}}}"),
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
                .then_ignore(just(FILE_BEGIN_MARK))
                .then(Self::argument_surrounded_by(BLOCK_DELIMITERS).padded())
                .then(prelude.repeated())
                .then(things_in_inputs.or(thing.clone()).repeated())
                .then_ignore(just(FILE_END_MARK).padded())
                .then(Self::argument_surrounded_by(BLOCK_DELIMITERS).padded())
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
                                format!("Mismatched file specifiers: {path} and {path_end}"),
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
        choice(SEGMENT_LEVELS.map(|(level, keyword)| {
            just::<char, &str, Simple<char>>(KEYWORD_PREFIX)
                .ignore_then(text::keyword(keyword))
                .to(level)
        }))
        .boxed()
    }
}
