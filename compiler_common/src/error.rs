use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

prelude! {}

/// Some value or an [`Error`].
pub type Res<T> = Result<T, Error>;

/// Some value or a [`TerminationError`].
pub type TRes<T> = Result<T, TerminationError>;

/// Termination of compilation error.
#[derive(Debug)]
pub struct TerminationError;

impl std::fmt::Display for TerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Termination Error")
    }
}

impl std::error::Error for TerminationError {}

/// Compilation errors enumeration.
///
/// [Error] enumeration is used during the compilation to alert the programmer of some errors in its
/// GRust program.
///
/// # Example
///
/// ```rust
/// # compiler_common::prelude! {}
/// let mut errors = vec![];
///
/// let name = String::from("unknown");
/// let loc = Location::default();
///
/// let error = Error::UnknownElement { name, loc };
/// errors.push(error);
/// ```
///
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Encountering an unknown element.
    UnknownElement {
        /// The unknown identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown signal.
    UnknownSignal {
        /// The unknown identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown node.
    UnknownNode {
        /// The unknown identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown interface.
    UnknownInterface {
        /// The unknown identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown type.
    UnknownType {
        /// The unknown identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown enumeration.
    UnknownEnumeration {
        /// The unknown enumeration identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Encountering an unknown field.
    UnknownField {
        /// The structure of the supposed field.
        structure_name: String,
        /// The unknown field.
        field_name: String,
        /// The error location.
        loc: Location,
    },
    /// A field is missing.
    MissingField {
        /// The structure of the missing field.
        structure_name: String,
        /// The missing field.
        field_name: String,
        /// The error location.
        loc: Location,
    },
    /// The index is out of bounds.
    IndexOutOfBounds {
        /// The error location.
        loc: Location,
    },
    /// Redefine an already defined element.
    AlreadyDefinedElement {
        /// The known identifier.
        name: String,
        /// The error location.
        loc: Location,
    },
    /// Incompatible type.
    IncompatibleType {
        /// Given type.
        given_type: Typ,
        /// Expected type.
        expected_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Incompatible tuple.
    IncompatibleTuple {
        /// The error location.
        loc: Location,
    },
    /// Incompatible match statements.
    IncompatibleMatchStatements {
        /// Expected number of statements.
        expected: usize,
        /// Received number of statements.
        received: usize,
        /// The error location.
        loc: Location,
    },
    /// Missing match statement.
    MissingMatchStatement {
        /// Ident of the missing statement in match.
        identifier: String,
        /// The error location
        loc: Location,
    },
    /// Not statement pattern error.
    NotStatementPattern {
        /// The error location.
        loc: Location,
    },
    /// Given inputs are not of the right number.
    ArityMismatch {
        /// The given number of inputs.
        input_count: usize,
        /// The expected number of inputs.
        arity: usize,
        /// The error location.
        loc: Location,
    },
    /// Calling an unknown output signal.
    UnknownOutputSignal {
        /// The node/component identifier.
        node_name: String,
        /// The unknown identifier.
        signal_name: String,
        /// The error location.
        loc: Location,
    },
    /// Expect constant expression.
    ExpectConstant {
        /// The error location.
        loc: Location,
    },
    /// Expect at least one input.
    ExpectInput {
        /// The error location.
        loc: Location,
    },
    /// Expect number type.
    ExpectNumber {
        /// Given type.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect abstraction with input type.
    ExpectAbstraction {
        /// Expected types as input for the abstraction.
        input_types: Vec<Typ>,
        /// Given type instead of the abstraction.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect option type.
    ExpectOption {
        /// Given type instead of the option.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect structure type.
    ExpectStructure {
        /// Given type instead of the structure.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect tuple type.
    ExpectTuple {
        /// Given type instead of the structure.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect array type.
    ExpectArray {
        /// Given type instead of the array.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect event type.
    ExpectEvent {
        /// Given type instead of the event.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect signal type.
    ExpectSignal {
        /// Given type instead of the signal.
        given_type: Typ,
        /// The error location.
        loc: Location,
    },
    /// Expect option pattern.
    ExpectOptionPattern {
        /// The error location.
        loc: Location,
    },
    /// Expect tuple pattern.
    ExpectTuplePattern {
        /// The error location.
        loc: Location,
    },
    /// Incompatible array length.
    IncompatibleLength {
        /// Given length.
        given_length: usize,
        /// Expected length.
        expected_length: usize,
        /// The error location.
        loc: Location,
    },
    /// Can not infer type.
    NoTypeInference {
        /// The error location.
        loc: Location,
    },
    /// Causality error.
    NotCausalSignal {
        /// Signal's name.
        signal: String,
        /// The error location.
        loc: Location,
    },
    /// Causality error.
    NotCausalNode {
        /// Node's name.
        node: String,
        /// The error location.
        loc: Location,
    },
    /// Unused signal error.
    UnusedSignal {
        /// Node's name.
        node: String,
        /// Signal's name.
        signal: String,
        /// The error location.
        loc: Location,
    },
}

impl Error {
    /// Transform the error into a diagnostic.
    ///
    /// This makes it possible to use [codespan_reporting] to print pretty errors.
    ///
    /// # Example
    /// ```rust
    /// # compiler_common::prelude! {}
    /// use codespan_reporting::{
    ///     files::SimpleFiles,
    ///     term::termcolor::{StandardStream, ColorChoice},
    ///     term,
    /// };
    ///
    /// let mut errors: Vec<Error> = vec![];
    /// let mut files = SimpleFiles::new();
    ///
    /// let file_id = files.add("file_test.gr", "a code without x...");
    /// let name = String::from("x");
    /// let loc = Location {
    ///     file_id,
    ///     range: 0..0,
    /// };
    ///
    /// let error = Error::UnknownElement { name, loc };
    /// errors.push(error);
    ///
    /// let writer = StandardStream::stderr(ColorChoice::Always);
    /// let config = term::Config::default();
    /// for error in &errors {
    ///     let writer = &mut writer.lock();
    ///     let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    /// }
    /// ```
    pub fn to_diagnostic(&self) -> Diagnostic<usize> {
        match self {
            Error::UnknownElement { name, loc } => Diagnostic::error()
                .with_message("unknown element")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("element '{name}' is not defined")]),
            Error::UnknownSignal { name, loc } => Diagnostic::error()
                .with_message("unknown signal")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("signal '{name}' is not defined")]),
            Error::UnknownNode { name, loc } => Diagnostic::error()
                .with_message("unknown node")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("node '{name}' is not defined")]),
            Error::UnknownInterface { name, loc } => Diagnostic::error()
                .with_message("unknown interface")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("interface '{name}' is not defined")]),
            Error::UnknownType { name, loc } => Diagnostic::error()
                .with_message("unknown type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("type '{name}' is not defined")]),
            Error::UnknownEnumeration { name, loc } => Diagnostic::error()
                .with_message("unknown enumeration")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!("enumeration '{name}' is not defined")]),
            Error::UnknownField {
                structure_name,
                field_name,
                loc,
            } => Diagnostic::error()
                .with_message("unknown field")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!(
                    "field '{field_name}' is not defined in structure '{structure_name}'"
                )]),
            Error::MissingField {
                structure_name,
                field_name,
                loc,
            } => Diagnostic::error()
                .with_message("missing field")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!(
                    "field '{field_name}' is missing in structure '{structure_name}' instantiation"
                )]),
            Error::IndexOutOfBounds { loc } => Diagnostic::error()
                .with_message("index out of bounds")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!("the index is out of bounds")]),
            Error::AlreadyDefinedElement { name, loc } => Diagnostic::error()
                .with_message("duplicated element")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("already defined")
                ])
                .with_notes(vec![format!(
                    "element '{name}' is already defined, please choose another name"
                )]),
            Error::IncompatibleType {
                given_type,
                expected_type,
                loc,
            } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected '{expected_type}' but '{given_type}' was given"
                )]),
            Error::IncompatibleTuple { loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())
                    .with_message("incompatible tuple type")]),
            Error::IncompatibleMatchStatements {
                expected,
                received,
                loc,
            } => Diagnostic::error()
                .with_message("incompatible number of statements in 'match'")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())
                    .with_message("incompatible statements")])
                .with_notes(vec![format!(
                    "expected {expected} statement{} but {received} statement{} {} given",
                    if expected < &2 { "" } else { "s" },
                    if received < &2 { "" } else { "s" },
                    if received < &2 { "was" } else { "were" }
                )]),
            Error::MissingMatchStatement { identifier, loc } => Diagnostic::error()
                .with_message("missing statement in 'match'")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())
                    .with_message("missing statement")])
                .with_notes(vec![format!(
                    "expected '{identifier}' to be defined in the match arm"
                )]),
            Error::NotStatementPattern { loc } => Diagnostic::error()
                .with_message("pattern error")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())
                    .with_message("not a statement pattern")]),
            Error::ArityMismatch {
                input_count,
                arity,
                loc,
            } => Diagnostic::error()
                .with_message("incompatible number of inputs")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())
                    .with_message("wrong number of inputs")])
                .with_notes(vec![format!(
                    "expected {arity} input{} but {input_count} input{} {} given",
                    if arity < &2 { "" } else { "s" },
                    if input_count < &2 { "" } else { "s" },
                    if input_count < &2 { "was" } else { "were" }
                )]),
            Error::UnknownOutputSignal {
                node_name,
                signal_name,
                loc,
            } => Diagnostic::error()
                .with_message("unknown output signal")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("unknown")
                ])
                .with_notes(vec![format!(
                    "signal '{signal_name}' is not an output of '{node_name}'"
                )]),
            Error::ExpectConstant { loc } => Diagnostic::error()
                .with_message("incompatible expression")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("not constant")
                ])
                .with_notes(vec![format!("expected a constant expression")]),
            Error::ExpectInput { loc } => Diagnostic::error()
                .with_message("missing inputs")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("empty")
                ])
                .with_notes(vec![format!("expected at least one input")]),
            Error::ExpectNumber { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected 'int' or 'float' but '{given_type}' was given"
                )]),
            Error::ExpectAbstraction {
                input_types,
                given_type,
                loc,
            } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected function type of the form '({}) -> t' but '{given_type}' was given",
                    input_types
                        .into_iter()
                        .map(|input_type| input_type.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )]),
            Error::ExpectOption { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected option type of the form 't?' but '{given_type}' was given"
                )]),
            Error::ExpectStructure { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected structure type but '{given_type}' was given"
                )]),
            Error::ExpectTuple { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected tuple type but '{given_type}' was given"
                )]),
            Error::ExpectArray { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected array type but '{given_type}' was given"
                )]),
            Error::ExpectEvent { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected event type but '{given_type}' was given"
                )]),
            Error::ExpectSignal { given_type, loc } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong type")
                ])
                .with_notes(vec![format!(
                    "expected signal type but '{given_type}' was given"
                )]),
            Error::ExpectOptionPattern { loc } => Diagnostic::error()
                .with_message("incompatible pattern")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong pattern")
                ])
                .with_notes(vec![format!(
                    "expected option pattern of the form 'some(p)'"
                )]),
            Error::ExpectTuplePattern { loc } => Diagnostic::error()
                .with_message("incompatible pattern")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong pattern")
                ])
                .with_notes(vec![format!(
                    "expected tuple pattern of the form '(p1, p2, ...)'"
                )]),
            Error::IncompatibleLength {
                given_length,
                expected_length,
                loc,
            } => Diagnostic::error()
                .with_message("incompatible array length")
                .with_labels(vec![
                    Label::primary(loc.file_id, loc.range.clone()).with_message("wrong length")
                ])
                .with_notes(vec![format!(
                    "\
                        expected array of length '{expected_length}' \
                        but an array of length '{given_length}' was given\
                    "
                )]),
            Error::NoTypeInference { loc } => Diagnostic::error()
                .with_message("can not infer type")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!("please explicit type")]),
            Error::NotCausalSignal { signal, loc } => Diagnostic::error()
                .with_message("not causal")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!("signal '{signal}' depends on itself")]),
            Error::NotCausalNode { node, loc } => Diagnostic::error()
                .with_message("not causal")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!("node '{node}' depends on itself")]),
            Error::UnusedSignal { node, signal, loc } => Diagnostic::error()
                .with_message("unused signal")
                .with_labels(vec![Label::primary(loc.file_id, loc.range.clone())])
                .with_notes(vec![format!(
                    "signal '{signal}' in node '{node}' in not used"
                )]),
        }
    }

    /// Display a list of errors in `stderr`.
    pub fn display_all(errors: &Vec<Error>, files: &SimpleFiles<&str, String>) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = term::Config::default();
        for error in errors {
            let writer = &mut writer.lock();
            let _ = term::emit(writer, &config, files, &error.to_diagnostic());
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnknownElement { name, .. } => write!(f, "unknown element `{name}`"),
            Error::UnknownSignal { name, .. } => write!(f, "unknown signal `{name}`"),
            Error::UnknownNode { name, .. } => write!(f, "unknown node `{name}`"),
            Error::UnknownInterface { name, .. } => write!(f, "unknown interface `{name}`"),
            Error::UnknownType { name, .. } => write!(f, "unknown type `{name}`"),
            Error::UnknownEnumeration { name, .. } => write!(f, "unknown enumeration `{name}`"),
            Error::UnknownField {
                structure_name,
                field_name,
                ..
            } => write!(f, "unknown field `{structure_name}::{field_name}"),
            Error::MissingField {
                structure_name,
                field_name,
                ..
            } => write!(f, "missing field `{structure_name}::{field_name}"),
            Error::IndexOutOfBounds { .. } => write!(f, "index out of bounds"),
            Error::AlreadyDefinedElement { name, .. } => {
                write!(f, "trying to redefine element `{name}`")
            }
            Error::IncompatibleType {
                given_type,
                expected_type,
                ..
            } => write!(
                f,
                "type mismatch: got `{given_type}`, expected `{expected_type}`"
            ),
            Error::IncompatibleTuple { .. } => write!(f, "incompatible tuple"),
            Error::IncompatibleMatchStatements {
                expected, received, ..
            } => write!(
                f,
                "incompatible match statements: got {}, expected {}",
                received, expected
            ),
            Error::MissingMatchStatement { identifier, .. } => {
                write!(f, "missing match statement for `{identifier}`")
            }
            Error::NotStatementPattern { .. } => write!(f, "not statement pattern"),
            Error::ArityMismatch {
                input_count, arity, ..
            } => write!(
                f,
                "arity mismatch: got {} input{}, expected {}",
                input_count,
                plural(*input_count),
                arity
            ),
            Error::UnknownOutputSignal {
                node_name,
                signal_name,
                ..
            } => write!(
                f,
                "unknown output signal in node `{}`: `{}`",
                node_name, signal_name
            ),
            Error::ExpectConstant { .. } => write!(f, "expected constant"),
            Error::ExpectInput { .. } => write!(f, "expected input"),
            Error::ExpectNumber { given_type, .. } => {
                write!(f, "expected a number, got `{}`", given_type)
            }
            Error::ExpectAbstraction { .. } => write!(f, "expected abstraction"),
            Error::ExpectOption { .. } => write!(f, "expected option"),
            Error::ExpectStructure { .. } => write!(f, "expected structure"),
            Error::ExpectTuple { .. } => write!(f, "expected tuple"),
            Error::ExpectArray { .. } => write!(f, "expected array"),
            Error::ExpectEvent { .. } => write!(f, "expected event"),
            Error::ExpectSignal { .. } => write!(f, "expected signal"),
            Error::ExpectOptionPattern { .. } => write!(f, "expected option pattern"),
            Error::ExpectTuplePattern { .. } => write!(f, "expected tuple pattern"),
            Error::IncompatibleLength { .. } => write!(f, "incompatible Length"),
            Error::NoTypeInference { .. } => write!(f, "no type inference"),
            Error::NotCausalSignal { .. } => write!(f, "not causal signal"),
            Error::NotCausalNode { .. } => write!(f, "not causal node"),
            Error::UnusedSignal { .. } => write!(f, "unused signal"),
        }
    }
}

impl std::error::Error for Error {}
