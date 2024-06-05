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
/// let location = Location::default();
///
/// let error = Error::UnknownElement { name, location };
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
        location: Location,
    },
    /// Encountering an unknown signal.
    UnknownSignal {
        /// The unknown identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Encountering an unknown node.
    UnknownNode {
        /// The unknown identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Encountering an unknown interface.
    UnknownInterface {
        /// The unknown identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Encountering an unknown type.
    UnknownType {
        /// The unknown identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Encountering an unknown enumeration.
    UnknownEnumeration {
        /// The unknown enumeration identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Encountering an unknown field.
    UnknownField {
        /// The structure of the supposed field.
        structure_name: String,
        /// The unknow field.
        field_name: String,
        /// The error location.
        location: Location,
    },
    /// A field is missing.
    MissingField {
        /// The structure of the missing field.
        structure_name: String,
        /// The missing field.
        field_name: String,
        /// The error location.
        location: Location,
    },
    /// The index is out of bounds.
    IndexOutOfBounds {
        /// The error location.
        location: Location,
    },
    /// Redefine an already defined element.
    AlreadyDefinedElement {
        /// The known identifier.
        name: String,
        /// The error location.
        location: Location,
    },
    /// Incompatible type.
    IncompatibleType {
        /// Given type.
        given_type: Typ,
        /// Expected type.
        expected_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Incompatible tuple.
    IncompatibleTuple {
        /// The error location.
        location: Location,
    },
    /// Incompatible match statements.
    IncompatibleMatchStatements {
        /// Expected number of statements.
        expected: usize,
        /// Received number of statements.
        received: usize,
        /// The error location.
        location: Location,
    },
    /// Missing match statement.
    MissingMatchStatement {
        /// Ident of the missing statement in match.
        identifier: String,
        /// The error location
        location: Location,
    },
    /// Not statement pattern error.
    NotStatementPattern {
        /// The error location.
        location: Location,
    },
    /// Given inputs are not of the right number.
    IncompatibleInputsNumber {
        /// The given number of inputs.
        given_inputs_number: usize,
        /// The expected number of inputs.
        expected_inputs_number: usize,
        /// The error location.
        location: Location,
    },
    /// Calling an unknown output signal.
    UnknownOuputSignal {
        /// The node/component identifier.
        node_name: String,
        /// The unknow identifier.
        signal_name: String,
        /// The error location.
        location: Location,
    },
    /// Expect constant expression.
    ExpectConstant {
        /// The error location.
        location: Location,
    },
    /// Expect at least one input.
    ExpectInput {
        /// The error location.
        location: Location,
    },
    /// Expect number type.
    ExpectNumber {
        /// Given type.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect abstraction with input type.
    ExpectAbstraction {
        /// Expected types as input for the abstraction.
        input_types: Vec<Typ>,
        /// Given type instead of the abstraction.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect option type.
    ExpectOption {
        /// Given type instead of the option.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect structure type.
    ExpectStructure {
        /// Given type instead of the structure.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect tuple type.
    ExpectTuple {
        /// Given type instead of the structure.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect array type.
    ExpectArray {
        /// Given type instead of the array.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect event type.
    ExpectEvent {
        /// Given type instead of the event.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect signal type.
    ExpectSignal {
        /// Given type instead of the signal.
        given_type: Typ,
        /// The error location.
        location: Location,
    },
    /// Expect option pattern.
    ExpectOptionPattern {
        /// The error location.
        location: Location,
    },
    /// Expect tuple pattern.
    ExpectTuplePattern {
        /// The error location.
        location: Location,
    },
    /// Incompatible array length.
    IncompatibleLength {
        /// Given length.
        given_length: usize,
        /// Expected length.
        expected_length: usize,
        /// The error location.
        location: Location,
    },
    /// Can not infer type.
    NoTypeInference {
        /// The error location.
        location: Location,
    },
    /// Causality error.
    NotCausalSignal {
        /// Signal's name.
        signal: String,
        /// The error location.
        location: Location,
    },
    /// Causality error.
    NotCausalNode {
        /// Node's name.
        node: String,
        /// The error location.
        location: Location,
    },
    /// Unused signal error.
    UnusedSignal {
        /// Node's name.
        node: String,
        /// Signal's name.
        signal: String,
        /// The error location.
        location: Location,
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
    /// let location = Location {
    ///     file_id,
    ///     range: 0..0,
    /// };
    ///
    /// let error = Error::UnknownElement { name, location };
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
            Error::UnknownElement { name, location } => Diagnostic::error()
                .with_message("unknown element")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("element '{name}' is not defined")
                ]
            ),
            Error::UnknownSignal { name, location } => Diagnostic::error()
                .with_message("unknown signal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("signal '{name}' is not defined")
                ]
            ),
            Error::UnknownNode { name, location } => Diagnostic::error()
                .with_message("unknown node")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("node '{name}' is not defined")
                ]
            ),
            Error::UnknownInterface { name, location } => Diagnostic::error()
                .with_message("unknown interface")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("interface '{name}' is not defined")
                ]
            ),
            Error::UnknownType { name, location } => Diagnostic::error()
                .with_message("unknown type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("type '{name}' is not defined")
                ]
            ),
            Error::UnknownEnumeration { name, location } => Diagnostic::error()
                .with_message("unknown enumeration")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("enumeration '{name}' is not defined")
                ]
            ),
            Error::UnknownField { structure_name, field_name, location } => Diagnostic::error()
                .with_message("unknown field")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("field '{field_name}' is not defined in structure '{structure_name}'")
                ]
            ),
            Error::MissingField { structure_name, field_name, location } => Diagnostic::error()
                .with_message("missing field")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("field '{field_name}' is missing in structure '{structure_name}' instantiation")
                ]
            ),
            Error::IndexOutOfBounds { location } => Diagnostic::error()
                .with_message("index out of bounds")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("the index is out of bounds")
                ]
            ),
            Error::AlreadyDefinedElement { name, location } => Diagnostic::error()
                .with_message("duplicated element")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("already defined")
                ])
                .with_notes(vec![
                    format!("element '{name}' is already defined, please choose another name")
                ]
            ),
            Error::IncompatibleType { given_type, expected_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expected '{expected_type}' but '{given_type}' was given")
                ]
            ),
            Error::IncompatibleTuple {  location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("incompatible tuple type")
                ]),
            Error::IncompatibleMatchStatements { expected, received, location } => Diagnostic::error()
                .with_message("incompatible number of statements in 'match'")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("incompatible statements")
                ])
                .with_notes(vec![
                    format!(
                        "expected {expected} statement{} but {received} statement{} {} given",
                        if expected < &2 {""} else {"s"},
                        if received < &2 {""} else {"s"},
                        if received < &2 {"was"} else {"were"}
                    )
                ]
            ),
            Error::MissingMatchStatement { identifier, location } => Diagnostic::error()
                .with_message("missing statement in 'match'")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("missing statement")
                ])
                .with_notes(vec![
                    format!(
                        "expected '{identifier}' to be defined in the match arm"
                    )
                ]
            ),
            Error::NotStatementPattern {  location } => Diagnostic::error()
                .with_message("pattern error")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("not a statement pattern")
                ]),
            Error::IncompatibleInputsNumber { given_inputs_number, expected_inputs_number, location } => Diagnostic::error()
                .with_message("incompatible number of inputs")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong number of inputs")
                ])
                .with_notes(vec![
                    format!(
                        "expected {expected_inputs_number} input{} but {given_inputs_number} input{} {} given",
                        if expected_inputs_number < &2 {""} else {"s"},
                        if given_inputs_number < &2 {""} else {"s"},
                        if given_inputs_number < &2 {"was"} else {"were"}
                    )
                ]
            ),
            Error::UnknownOuputSignal { node_name, signal_name, location } => Diagnostic::error()
                .with_message("unknown output signal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("unknown")
                ])
                .with_notes(vec![
                    format!("signal '{signal_name}' is not an output of '{node_name}'")
                ]
            ),
            Error::ExpectConstant { location } => Diagnostic::error()
                .with_message("incompatible expression")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("not constant")
                ])
                .with_notes(vec![
                    format!("expect a constant expression")
                ]
            ),
            Error::ExpectInput { location } => Diagnostic::error()
                .with_message("missing inputs")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("empty")
                ])
                .with_notes(vec![
                    format!("expect at least one input")
                ]
            ),
            Error::ExpectNumber { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expected 'int' or 'float' but '{given_type}' was given")
                ]
            ),
            Error::ExpectAbstraction { input_types, given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect function type of the form '({}) -> t' but '{given_type}' was given",
                    input_types.into_iter().map(|input_type| input_type.to_string()).collect::<Vec<_>>().join(", ")
                )
                ]
            ),
            Error::ExpectOption { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect option type of the form 't?' but '{given_type}' was given")
                ]
            ),
            Error::ExpectStructure { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect structure type but '{given_type}' was given")
                ]
            ),
            Error::ExpectTuple { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect tuple type but '{given_type}' was given")
                ]
            ),
            Error::ExpectArray { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect array type but '{given_type}' was given")
                ]
            ),
            Error::ExpectEvent { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect event type but '{given_type}' was given")
                ]
            ),
            Error::ExpectSignal { given_type, location } => Diagnostic::error()
                .with_message("incompatible type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect signal type but '{given_type}' was given")
                ]
            ),
            Error::ExpectOptionPattern { location } => Diagnostic::error()
                .with_message("incompatible pattern")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong pattern")
                ])
                .with_notes(vec![
                    format!("expect option pattern of the form 'some(p)'")
                ]
            ),
            Error::ExpectTuplePattern { location } => Diagnostic::error()
                .with_message("incompatible pattern")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong pattern")
                ])
                .with_notes(vec![
                    format!("expect tuple pattern of the form '(p1, p2, ...)'")
                ]
            ),
            Error::IncompatibleLength { given_length, expected_length, location } => Diagnostic::error()
                .with_message("incompatible array lenght")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong length")
                ])
                .with_notes(vec![
                    format!("expect array of length '{expected_length}' but an array of length '{given_length}' was given")
                ]
            ),
            Error::NoTypeInference { location } => Diagnostic::error()
                .with_message("can not infere type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("please explicit type")
                ]
            ),
            Error::NotCausalSignal { signal, location } => Diagnostic::error()
                .with_message("not causal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("signal '{signal}' depends on itself")
                ]
            ),
            Error::NotCausalNode { node,  location } => Diagnostic::error()
                .with_message("not causal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("node '{node}' depends on itself")
                ]
            ),
            Error::UnusedSignal { node, signal, location } => Diagnostic::error()
                .with_message("unused signal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("signal '{signal}' in node '{node}' in not used")
                ]
            ),
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
            Error::UnknownElement { .. } => write!(f, "Unknown Element"),
            Error::UnknownSignal { .. } => write!(f, "Unknown Signal"),
            Error::UnknownNode { .. } => write!(f, "Unknown Node"),
            Error::UnknownInterface { .. } => write!(f, "Unknown Interface"),
            Error::UnknownType { .. } => write!(f, "Unknown Type"),
            Error::UnknownEnumeration { .. } => write!(f, "Unknown Enumeration"),
            Error::UnknownField { .. } => write!(f, "Unknown Field"),
            Error::MissingField { .. } => write!(f, "Missing Field"),
            Error::IndexOutOfBounds { .. } => write!(f, "Index Out Of Bounds"),
            Error::AlreadyDefinedElement { .. } => write!(f, "Already Defined Element"),
            Error::IncompatibleType { .. } => write!(f, "Incompatible Type"),
            Error::IncompatibleTuple { .. } => write!(f, "Incompatible Tuple"),
            Error::IncompatibleMatchStatements { .. } => write!(f, "Incompatible Match Statements"),
            Error::MissingMatchStatement { .. } => write!(f, "Missing Match Statement"),
            Error::NotStatementPattern { .. } => write!(f, "Not Statement Pattern"),
            Error::IncompatibleInputsNumber { .. } => write!(f, "Incompatible Inputs Number"),
            Error::UnknownOuputSignal { .. } => write!(f, "Unknown Output Signal"),
            Error::ExpectConstant { .. } => write!(f, "Expect Constant"),
            Error::ExpectInput { .. } => write!(f, "Expect Input"),
            Error::ExpectNumber { .. } => write!(f, "Expect Number"),
            Error::ExpectAbstraction { .. } => write!(f, "Expect Abstraction"),
            Error::ExpectOption { .. } => write!(f, "Expect Option"),
            Error::ExpectStructure { .. } => write!(f, "Expect Structure"),
            Error::ExpectTuple { .. } => write!(f, "Expect Tuple"),
            Error::ExpectArray { .. } => write!(f, "Expect Array"),
            Error::ExpectEvent { .. } => write!(f, "Expect Event"),
            Error::ExpectSignal { .. } => write!(f, "Expect Signal"),
            Error::ExpectOptionPattern { .. } => write!(f, "Expect Option Pattern"),
            Error::ExpectTuplePattern { .. } => write!(f, "Expect Tuple Pattern"),
            Error::IncompatibleLength { .. } => write!(f, "Incompatible Length"),
            Error::NoTypeInference { .. } => write!(f, "No Type Inference"),
            Error::NotCausalSignal { .. } => write!(f, "Not Causal Signal"),
            Error::NotCausalNode { .. } => write!(f, "Not Causal Node"),
            Error::UnusedSignal { .. } => write!(f, "Unused Signal"),
        }
    }
}

impl std::error::Error for Error {}
