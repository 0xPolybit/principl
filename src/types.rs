use std::fmt;

/// A resolved Princi type. `String`, `Class`, and `List` are managed-reference
/// types; they are distinct from inline values such as structs. Princi has no
/// raw-pointer type. `Any`, `Error`, function, range, generic-parameter, and
/// generic-instance types are compiler representations; generic declarations
/// and instances are checked before the native backend specializes them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Class(String),
    Struct(String),
    TypeParameter(GenericTypeParameter),
    GenericInstance {
        constructor: GenericTypeConstructor,
        arguments: Vec<Type>,
    },
    Function(FunctionType),
    Range(Box<Type>),
    Any,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericTypeParameter {
    /// Unique within a compilation; names alone do not identify a parameter.
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericTypeConstructor {
    List,
    Class(String),
    Struct(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub type_parameters: Vec<GenericTypeParameter>,
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self {
            type_parameters: Vec::new(),
            parameters,
            return_type: Box::new(return_type),
        }
    }

    pub fn generic(
        type_parameters: Vec<GenericTypeParameter>,
        parameters: Vec<Type>,
        return_type: Type,
    ) -> Self {
        Self {
            type_parameters,
            parameters,
            return_type: Box::new(return_type),
        }
    }
}

impl Type {
    pub fn list(element: Type) -> Self {
        Self::GenericInstance {
            constructor: GenericTypeConstructor::List,
            arguments: vec![element],
        }
    }

    pub fn list_element(&self) -> Option<&Type> {
        match self {
            Type::GenericInstance {
                constructor: GenericTypeConstructor::List,
                arguments,
            } => arguments.first(),
            _ => None,
        }
    }

    pub fn into_list_element(self) -> Option<Type> {
        match self {
            Type::GenericInstance {
                constructor: GenericTypeConstructor::List,
                mut arguments,
            } if arguments.len() == 1 => Some(arguments.remove(0)),
            _ => None,
        }
    }

    pub fn generic_instance(constructor: GenericTypeConstructor, arguments: Vec<Type>) -> Self {
        Self::GenericInstance {
            constructor,
            arguments,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_))
            || matches!(
                self,
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::Struct(_),
                    ..
                }
            )
    }

    pub fn nominal_name(&self) -> Option<&str> {
        match self {
            Type::Class(name) | Type::Struct(name) => Some(name),
            Type::GenericInstance {
                constructor:
                    GenericTypeConstructor::Class(name) | GenericTypeConstructor::Struct(name),
                ..
            } => Some(name),
            _ => None,
        }
    }

    pub fn generic_arguments(&self) -> &[Type] {
        match self {
            Type::GenericInstance { arguments, .. } => arguments,
            _ => &[],
        }
    }

    /// Whether a value of `actual` can be used where this type is expected.
    /// Any is currently limited to compiler-internal built-in print typing.
    pub fn accepts(&self, actual: &Type) -> bool {
        match (self, actual) {
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::Any, Type::Void) | (Type::Void, Type::Any) => false,
            (Type::Any, _) => true,
            (_, Type::Any) => false,
            (
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::List,
                    arguments: expected,
                },
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::List,
                    arguments: actual,
                },
            ) if expected.len() == 1 && actual.len() == 1 => expected[0].accepts(&actual[0]),
            _ => self == actual,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Whether this value is an opaque reference managed by the runtime.
    /// Struct values can contain references but are copied inline themselves.
    pub fn is_managed_reference(&self) -> bool {
        matches!(self, Type::String | Type::Class(_))
            || matches!(
                self,
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::List | GenericTypeConstructor::Class(_),
                    ..
                }
            )
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
            Type::TypeParameter(parameter) => formatter.write_str(&parameter.name),
            Type::GenericInstance {
                constructor,
                arguments,
            } => {
                let name = match constructor {
                    GenericTypeConstructor::List => "List",
                    GenericTypeConstructor::Class(name) | GenericTypeConstructor::Struct(name) => {
                        name
                    }
                };
                write!(formatter, "{name}<{}>", display_types(arguments))
            }
            Type::Function(function) => {
                let parameters = function
                    .parameters
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let generic_parameters = if function.type_parameters.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        display_generic_parameters(&function.type_parameters)
                    )
                };
                write!(
                    formatter,
                    "{generic_parameters}({parameters}) -> {}",
                    function.return_type
                )
            }
            Type::Range(element) => write!(formatter, "Range<{element}>"),
            Type::Any => formatter.write_str("Any"),
            Type::Error => formatter.write_str("<error>"),
        }
    }
}

fn display_types(types: &[Type]) -> String {
    types
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_generic_parameters(parameters: &[GenericTypeParameter]) -> String {
    parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{FunctionType, GenericTypeConstructor, Type};

    #[test]
    fn displays_nested_types_and_function_signatures() {
        let list = Type::list(Type::Struct("Point".to_owned()));
        assert_eq!(list.to_string(), "List<Point>");

        let function = Type::Function(FunctionType::new(vec![Type::Int, list], Type::Bool));
        assert_eq!(function.to_string(), "(Int, List<Point>) -> Bool");
    }

    #[test]
    fn list_any_is_a_compatible_internal_annotation() {
        let any_list = Type::list(Type::Any);
        let int_list = Type::list(Type::Int);
        assert!(any_list.accepts(&int_list));
        assert!(!Type::Int.accepts(&Type::String));
        assert!(!Type::Int.accepts(&Type::Any));
        assert!(!Type::Any.accepts(&Type::Void));
    }

    #[test]
    fn distinguishes_managed_references_from_inline_values() {
        assert!(Type::String.is_managed_reference());
        assert!(Type::Class("User".to_owned()).is_managed_reference());
        assert!(Type::list(Type::Int).is_managed_reference());
        assert!(!Type::Int.is_managed_reference());
        assert!(!Type::Struct("Point".to_owned()).is_managed_reference());
    }

    #[test]
    fn displays_generic_nominal_instances_and_function_parameters() {
        let pair = Type::generic_instance(
            GenericTypeConstructor::Struct("Pair".to_owned()),
            vec![Type::String, Type::Float],
        );
        assert_eq!(pair.to_string(), "Pair<String, Float>");
    }
}
