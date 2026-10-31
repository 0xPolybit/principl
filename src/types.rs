use std::fmt;

/// A resolved Princi type. `String`, `Class`, `List`, and `Task` are managed-reference
/// types; they are distinct from inline values such as structs. An `Interface`
/// value is an inline pair containing a managed class reference and a dispatch
/// table reference. A `Trait` value uses the same pair; class values keep their
/// reference, while struct values are copied into managed storage when viewed
/// through a trait. Enum values are inline tagged aggregates: copying one
/// copies its payload slots, while managed-reference payloads remain shared
/// under their existing type rules. Princi has no raw-pointer type. `Any`,
/// `Error`, function, range, generic-parameter, and generic-instance types are
/// compiler representations; generic declarations and instances are checked
/// before the native backend specializes them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Class(String),
    Struct(String),
    Enum(String),
    Interface(String),
    Trait(String),
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
    /// Interface or trait contracts required of a concrete specialization.
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericTypeConstructor {
    List,
    Task,
    Class(String),
    Struct(String),
    Enum(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub type_parameters: Vec<GenericTypeParameter>,
    pub parameters: Vec<Type>,
    /// The source-level result type is `T`; calling an async function produces
    /// the compiler-managed `Task<T>` handle instead.
    pub is_async: bool,
    pub return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self::with_async(Vec::new(), parameters, return_type, false)
    }

    pub fn asynchronous(parameters: Vec<Type>, return_type: Type) -> Self {
        Self::with_async(Vec::new(), parameters, return_type, true)
    }

    pub fn with_async(
        type_parameters: Vec<GenericTypeParameter>,
        parameters: Vec<Type>,
        return_type: Type,
        is_async: bool,
    ) -> Self {
        Self {
            type_parameters,
            parameters,
            is_async,
            return_type: Box::new(return_type),
        }
    }

    pub fn generic(
        type_parameters: Vec<GenericTypeParameter>,
        parameters: Vec<Type>,
        return_type: Type,
    ) -> Self {
        Self::with_async(type_parameters, parameters, return_type, false)
    }

    pub fn generic_async(
        type_parameters: Vec<GenericTypeParameter>,
        parameters: Vec<Type>,
        return_type: Type,
    ) -> Self {
        Self::with_async(type_parameters, parameters, return_type, true)
    }

    /// Result observed by a call expression, distinct from the source-level
    /// function result used to check `return` statements in async bodies.
    pub fn call_result_type(&self) -> Type {
        let result = self.return_type.as_ref().clone();
        if self.is_async {
            Type::task(result)
        } else {
            result
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

    pub fn task(result: Type) -> Self {
        Self::GenericInstance {
            constructor: GenericTypeConstructor::Task,
            arguments: vec![result],
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

    pub fn is_enum(&self) -> bool {
        matches!(self, Type::Enum(_))
            || matches!(
                self,
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::Enum(_),
                    ..
                }
            )
    }

    pub fn nominal_name(&self) -> Option<&str> {
        match self {
            Type::Class(name)
            | Type::Struct(name)
            | Type::Enum(name)
            | Type::Interface(name)
            | Type::Trait(name) => Some(name),
            Type::GenericInstance {
                constructor:
                    GenericTypeConstructor::Class(name)
                    | GenericTypeConstructor::Struct(name)
                    | GenericTypeConstructor::Enum(name),
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
            (
                Type::Any,
                Type::GenericInstance {
                    constructor: GenericTypeConstructor::Task,
                    ..
                },
            ) => false,
            (Type::Any, _) => true,
            (_, Type::Any) => false,
            (
                Type::GenericInstance {
                    constructor: expected_constructor,
                    arguments: expected_arguments,
                },
                Type::GenericInstance {
                    constructor: actual_constructor,
                    arguments: actual_arguments,
                },
            ) if expected_constructor == actual_constructor
                && expected_arguments.len() == actual_arguments.len() =>
            {
                expected_arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| expected.accepts(actual))
            }
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
                    constructor: GenericTypeConstructor::List
                        | GenericTypeConstructor::Task
                        | GenericTypeConstructor::Class(_),
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
            Type::Class(name)
            | Type::Struct(name)
            | Type::Enum(name)
            | Type::Interface(name)
            | Type::Trait(name) => formatter.write_str(name),
            Type::TypeParameter(parameter) => formatter.write_str(&parameter.name),
            Type::GenericInstance {
                constructor,
                arguments,
            } => {
                let name = match constructor {
                    GenericTypeConstructor::List => "List",
                    GenericTypeConstructor::Task => "Task",
                    GenericTypeConstructor::Class(name)
                    | GenericTypeConstructor::Struct(name)
                    | GenericTypeConstructor::Enum(name) => name,
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
                if function.is_async {
                    write!(
                        formatter,
                        "async {generic_parameters}({parameters}) -> {}",
                        function.return_type
                    )
                } else {
                    write!(
                        formatter,
                        "{generic_parameters}({parameters}) -> {}",
                        function.return_type
                    )
                }
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
    fn generic_any_arguments_follow_one_shared_compatibility_rule() {
        let any_list = Type::list(Type::Any);
        let int_list = Type::list(Type::Int);
        assert!(any_list.accepts(&int_list));
        assert!(!int_list.accepts(&any_list));

        let any_box = Type::generic_instance(
            GenericTypeConstructor::Class("Box".to_owned()),
            vec![Type::Any],
        );
        let int_box = Type::generic_instance(
            GenericTypeConstructor::Class("Box".to_owned()),
            vec![Type::Int],
        );
        assert!(any_box.accepts(&int_box));
        assert!(!int_box.accepts(&any_box));

        let any_pair = Type::generic_instance(
            GenericTypeConstructor::Struct("Pair".to_owned()),
            vec![Type::Any, Type::String],
        );
        let pair = Type::generic_instance(
            GenericTypeConstructor::Struct("Pair".to_owned()),
            vec![Type::Int, Type::String],
        );
        assert!(any_pair.accepts(&pair));
        assert!(!any_pair.accepts(&Type::generic_instance(
            GenericTypeConstructor::Struct("Pair".to_owned()),
            vec![Type::Int, Type::Float],
        )));
        assert!(!any_pair.accepts(&Type::generic_instance(
            GenericTypeConstructor::Struct("OtherPair".to_owned()),
            vec![Type::Int, Type::String],
        )));
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
        assert!(!Type::Interface("Drawable".to_owned()).is_managed_reference());
        assert!(!Type::Trait("Named".to_owned()).is_managed_reference());
    }

    #[test]
    fn displays_generic_nominal_instances_and_function_parameters() {
        let pair = Type::generic_instance(
            GenericTypeConstructor::Struct("Pair".to_owned()),
            vec![Type::String, Type::Float],
        );
        assert_eq!(pair.to_string(), "Pair<String, Float>");
    }

    #[test]
    fn generic_enum_types_are_inline_values_not_managed_references() {
        let maybe = Type::generic_instance(
            GenericTypeConstructor::Enum("Maybe".to_owned()),
            vec![Type::String],
        );
        assert_eq!(maybe.to_string(), "Maybe<String>");
        assert!(maybe.is_enum());
        assert!(!maybe.is_struct());
        assert!(!maybe.is_managed_reference());
        assert!(!Type::Enum("Direction".to_owned()).is_managed_reference());
    }

    #[test]
    fn task_types_are_managed_and_async_signatures_expose_task_results() {
        let task = Type::task(Type::Int);
        assert_eq!(task.to_string(), "Task<Int>");
        assert!(task.is_managed_reference());
        assert!(!Type::Int.accepts(&task));
        assert!(!Type::Any.accepts(&task));
        assert!(task.accepts(&Type::task(Type::Int)));

        let parameter = super::GenericTypeParameter {
            id: 7,
            name: "T".to_owned(),
            bounds: Vec::new(),
        };
        let signature = FunctionType::generic_async(
            vec![parameter.clone()],
            vec![Type::TypeParameter(parameter.clone())],
            Type::TypeParameter(parameter),
        );
        assert!(signature.is_async);
        assert_eq!(
            signature.call_result_type(),
            Type::task(Type::TypeParameter(super::GenericTypeParameter {
                id: 7,
                name: "T".to_owned(),
                bounds: Vec::new(),
            }))
        );
        assert_eq!(Type::Function(signature).to_string(), "async <T>(T) -> T");
        assert_eq!(Type::task(Type::Void).to_string(), "Task<Void>");
    }
}
