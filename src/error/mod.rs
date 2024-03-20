use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use crate::common::{location::Location, r#type::Type};

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
/// [Error] enumeration is used during the compilation to alert
/// the programmer of some errors in its LanGRust program.
///
/// # Example
/// ```rust
/// use grustine::common::location::Location;
/// use grustine::error::Error;
///
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
    /// encountering an unknown element
    UnknownElement {
        /// the unknow identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// encountering an unknown signal
    UnknownSignal {
        /// the unknow identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// encountering an unknown node
    UnknownNode {
        /// the unknow identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// encountering an unknown type
    UnknownType {
        /// the unknow identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// encountering an unknown enumeration
    UnknownEnumeration {
        /// the unknow enumeration identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// encountering an unknown field
    UnknownField {
        /// the structure of the supposed field
        structure_name: String,
        /// the unknow field
        field_name: String,
        /// the error location
        location: Location,
    },
    /// a field is missing
    MissingField {
        /// the structure of the missing field
        structure_name: String,
        /// the missing field
        field_name: String,
        /// the error location
        location: Location,
    },
    /// the index is out of bounds
    IndexOutOfBounds {
        /// the error location
        location: Location,
    },
    /// component is called
    ComponentCall {
        /// name of the calle Component
        name: String,
        /// the error location
        location: Location,
    },
    /// redefine an already defined element
    AlreadyDefinedElement {
        /// the known identifier
        name: String,
        /// the error location
        location: Location,
    },
    /// incompatible type
    IncompatibleType {
        /// given type
        given_type: Type,
        /// expected type
        expected_type: Type,
        /// the error location
        location: Location,
    },
    /// given inputs are not of the right number
    IncompatibleInputsNumber {
        /// the given number of inputs
        given_inputs_number: usize,
        /// the expected number of inputs
        expected_inputs_number: usize,
        /// the error location
        location: Location,
    },
    /// expect constant expression
    ExpectConstant {
        /// the error location
        location: Location,
    },
    /// expect at least one input
    ExpectInput {
        /// the error location
        location: Location,
    },
    /// expect number type
    ExpectNumber {
        /// given type
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect abstraction with input type
    ExpectAbstraction {
        /// expected types as input for the abstraction
        input_types: Vec<Type>,
        /// given type instead of the abstraction
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect option type
    ExpectOption {
        /// given type instead of the option
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect structure type
    ExpectStructure {
        /// given type instead of the structure
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect tuple type
    ExpectTuple {
        /// given type instead of the structure
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect array type
    ExpectArray {
        /// given type instead of the array
        given_type: Type,
        /// the error location
        location: Location,
    },
    /// expect option pattern
    ExpectOptionPattern {
        /// the error location
        location: Location,
    },
    /// expect tuple pattern
    ExpectTuplePattern {
        /// the error location
        location: Location,
    },
    /// incompatible array length
    IncompatibleLength {
        /// given length
        given_length: usize,
        /// expected length
        expected_length: usize,
        /// the error location
        location: Location,
    },
    /// can not infere type
    NoTypeInference {
        /// the error location
        location: Location,
    },
    /// causality error
    NotCausalSignal {
        /// node's name
        node: String,
        /// signal's name
        signal: String,
        /// the error location
        location: Location,
    },
    /// causality error
    NotCausalNode {
        /// node's name
        node: String,
        /// the error location
        location: Location,
    },
    /// unused signal error
    UnusedSignal {
        /// node's name
        node: String,
        /// signal's name
        signal: String,
        /// the error location
        location: Location,
    },
}

impl Error {
    /// Transform the error into a diagnostic.
    ///
    /// This makes it possible to use [codespan_reporting] to print
    /// pretty errors.
    ///
    /// # Example
    /// ```rust
    /// use codespan_reporting::{
    ///     files::SimpleFiles,
    ///     term::termcolor::{StandardStream, ColorChoice},
    ///     term,
    /// };
    ///
    /// use grustine::common::location::Location;
    /// use grustine::error::Error;
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
            Error::ComponentCall { name, location } => Diagnostic::error()
                .with_message("component can not be called")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("'{name}' is a component, it can not be called")
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
            Error::NotCausalSignal { node, signal, location } => Diagnostic::error()
                .with_message("not causal")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                ])
                .with_notes(vec![
                    format!("signal '{signal}' depends on itself in node '{node}'")
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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnknownElement { .. } => write!(f, "Unknown Element"),
            Error::UnknownSignal { .. } => write!(f, "Unknown Signal"),
            Error::UnknownNode { .. } => write!(f, "Unknown Node"),
            Error::UnknownType { .. } => write!(f, "Unknown Type"),
            Error::UnknownEnumeration { .. } => write!(f, "Unknown Enumeration"),
            Error::UnknownField { .. } => write!(f, "Unknown Field"),
            Error::MissingField { .. } => write!(f, "Missing Field"),
            Error::IndexOutOfBounds { .. } => write!(f, "Index Out Of Bounds"),
            Error::ComponentCall { .. } => write!(f, "Component Identifier"),
            Error::AlreadyDefinedElement { .. } => write!(f, "Already Defined Type"),
            Error::IncompatibleType { .. } => write!(f, "Incompatible Type"),
            Error::IncompatibleInputsNumber { .. } => write!(f, "Incompatible Inputs Number"),
            Error::ExpectConstant { .. } => write!(f, "Expect Constant"),
            Error::ExpectInput { .. } => write!(f, "Expect Input"),
            Error::ExpectNumber { .. } => write!(f, "Expect Number"),
            Error::ExpectAbstraction { .. } => write!(f, "Expect Abstraction"),
            Error::ExpectOption { .. } => write!(f, "Expect Option"),
            Error::ExpectStructure { .. } => write!(f, "Expect Structure"),
            Error::ExpectTuple { .. } => write!(f, "Expect Tuple"),
            Error::ExpectArray { .. } => write!(f, "Expect Array"),
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

/// Display errors in terminal.
pub fn display(errors: &Vec<Error>, files: &SimpleFiles<&str, String>) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, files, &error.to_diagnostic());
    }
}
