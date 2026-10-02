use std::fmt;

/// A resolved Princi type. `Any`, `Error`, function, and range types are
/// compiler-internal; source declarations can name the primitive and declared
/// class/struct types, plus the unparameterized `List` annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Class(String),
    Struct(String),
    List(Box<Type>),
    Function(FunctionType),
    Range(Box<Type>),
    Any,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self {
            parameters,
            return_type: Box::new(return_type),
        }
    }
}

impl Type {
    /// Whether a value of `actual` can be used where this type is expected.
    /// `Any` is currently limited to the built-in print parameter and the
    /// element type of a bare `List` annotation.
    pub fn accepts(&self, actual: &Type) -> bool {
        match (self, actual) {
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::Any, Type::Void) | (Type::Void, Type::Any) => false,
            (Type::Any, _) => true,
            (_, Type::Any) => false,
            (Type::List(expected), Type::List(actual)) => expected.accepts(actual),
            _ => self == actual,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => formatter.write_str("Int"),
            Type::Float => formatter.write_str("Float"),
            Type::Bool => formatter.write_str("Bool"),
            Type::String => formatter.write_str("String"),
            Type::Void => formatter.write_str("Void"),
            Type::Class(name) | Type::Struct(name) => formatter.write_str(name),
            Type::List(element) => write!(formatter, "List<{element}>"),
            Type::Function(function) => {
                let parameters = function
                    .parameters
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(formatter, "({parameters}) -> {}", function.return_type)
            }
            Type::Range(element) => write!(formatter, "Range<{element}>"),
            Type::Any => formatter.write_str("Any"),
            Type::Error => formatter.write_str("<error>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FunctionType, Type};

    #[test]
    fn displays_nested_types_and_function_signatures() {
        let list = Type::List(Box::new(Type::Struct("Point".to_owned())));
        assert_eq!(list.to_string(), "List<Point>");

        let function = Type::Function(FunctionType::new(vec![Type::Int, list], Type::Bool));
        assert_eq!(function.to_string(), "(Int, List<Point>) -> Bool");
    }

    #[test]
    fn list_any_is_a_compatible_internal_annotation() {
        let any_list = Type::List(Box::new(Type::Any));
        let int_list = Type::List(Box::new(Type::Int));
        assert!(any_list.accepts(&int_list));
        assert!(!Type::Int.accepts(&Type::String));
        assert!(!Type::Int.accepts(&Type::Any));
        assert!(!Type::Any.accepts(&Type::Void));
    }
}
