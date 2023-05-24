use codespan_reporting::diagnostic::{Diagnostic, Label};

use crate::ast::{
    location::Location,
    type_system::Type,
};

/// Compilation errors enumeration.
/// 
/// [Error] enumeration is used during the compilation to alert
/// the programmer of some errors in its LanGRust program.
/// 
/// # Example
/// ```rust
/// use grustine::error::Error;
/// use grustine::ast::location::Location;
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// encountering an unknown element
    UnknownElement {
        /// the unknow identifier
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
    /// expect abstraction with input type
    ExpectAbstraction {
        /// expected type as input for the abstraction
        input_type: Type,
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
    /// can not infere type
    NoTypeInference {
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
    /// use grustine::error::Error;
    /// use grustine::ast::location::Location;
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
            Error::ExpectAbstraction { input_type, given_type, location } => Diagnostic::error()
                .with_message("expect abstraction")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("wrong type")
                ])
                .with_notes(vec![
                    format!("expect abstraction of the form '{input_type} -> t' but '{given_type}' was given")
                ]
            ),
            Error::ExpectOption { given_type, location } => Diagnostic::error()
                .with_message("expect option type")
                .with_labels(vec![
                    Label::primary(location.file_id, location.range.clone())
                        .with_message("not option type")
                ])
                .with_notes(vec![
                    format!("expect option type of the form 't?' but '{given_type}' was given")
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
        }
    }
}
