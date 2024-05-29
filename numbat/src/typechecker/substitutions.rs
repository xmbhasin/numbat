use thiserror::Error;

use crate::type_variable::TypeVariable;
use crate::typed_ast::{DType, Expression, StructInfo, Type};
use crate::Statement;

#[derive(Debug, Clone)]
pub struct Substitution(pub Vec<(TypeVariable, Type)>);

impl Substitution {
    pub fn empty() -> Substitution {
        Substitution(vec![])
    }

    pub fn single(v: TypeVariable, t: Type) -> Substitution {
        Substitution(vec![(v, t)])
    }

    pub fn lookup(&self, v: &TypeVariable) -> Option<&Type> {
        self.0.iter().find(|(var, _)| var == v).map(|(_, t)| t)
    }

    // pub fn pretty_print(&self) -> String {
    //     self.0
    //         .iter()
    //         .map(|(v, t)| format!("  {} := {}", v.name(), t))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    // }

    pub fn extend(&mut self, other: Substitution) {
        for (_, t) in &mut self.0 {
            t.apply(&other).unwrap(); // TODO: is the unwrap okay here?
        }
        self.0.extend(other.0);
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SubstitutionError {
    #[error("Used non-dimension type in a dimension expression: {0}")]
    SubstitutedNonDTypeWithinDType(Type),
}

pub trait ApplySubstitution {
    fn apply(&mut self, substitution: &Substitution) -> Result<(), SubstitutionError>;
}

impl ApplySubstitution for Type {
    fn apply(&mut self, s: &Substitution) -> Result<(), SubstitutionError> {
        match self {
            Type::TVar(v) => {
                if let Some(type_) = s.lookup(v) {
                    *self = type_.clone();
                }
                Ok(())
            }
            Type::Dimension(dtype) => dtype.apply(s),
            Type::Boolean => Ok(()),
            Type::String => Ok(()),
            Type::DateTime => Ok(()),
            Type::Fn(param_types, return_type) => {
                for param_type in param_types {
                    param_type.apply(s)?;
                }
                return_type.apply(s)
            }
            Type::Struct(info) => {
                for (_, field_type) in info.fields.values_mut() {
                    field_type.apply(s)?;
                }
                Ok(())
            }
            Type::List(element_type) => element_type.apply(s),
        }
    }
}

impl ApplySubstitution for DType {
    fn apply(&mut self, _s: &Substitution) -> Result<(), SubstitutionError> {
        // TODO
        Ok(())
    }
}

impl ApplySubstitution for StructInfo {
    fn apply(&mut self, s: &Substitution) -> Result<(), SubstitutionError> {
        for (_, field_type) in self.fields.values_mut() {
            field_type.apply(s)?;
        }
        Ok(())
    }
}

impl ApplySubstitution for Expression {
    fn apply(&mut self, s: &Substitution) -> Result<(), SubstitutionError> {
        match self {
            Expression::Scalar(_, _, type_) => type_.apply(s),
            Expression::Identifier(_, _, type_) => type_.apply(s),
            Expression::UnitIdentifier(_, _, _, _, type_) => type_.apply(s),
            Expression::UnaryOperator(_, _, expr, type_) => {
                expr.apply(s)?;
                type_.apply(s)
            }
            Expression::BinaryOperator(_, _, lhs, rhs, type_) => {
                lhs.apply(s)?;
                rhs.apply(s)?;
                type_.apply(s)
            }
            Expression::BinaryOperatorForDate(_, _, lhs, rhs, type_) => {
                lhs.apply(s)?;
                rhs.apply(s)?;
                type_.apply(s)
            }
            Expression::FunctionCall(_, _, _, arguments, return_type) => {
                for arg in arguments {
                    arg.apply(s)?;
                }
                return_type.apply(s)
            }
            Expression::CallableCall(_, callable, arguments, return_type) => {
                callable.apply(s)?;
                for arg in arguments {
                    arg.apply(s)?;
                }
                return_type.apply(s)
            }
            Expression::Boolean(_, _) => Ok(()),
            Expression::Condition(_, if_, then_, else_) => {
                if_.apply(s)?;
                then_.apply(s)?;
                else_.apply(s)
            }
            Expression::String(_, _) => Ok(()),
            Expression::InstantiateStruct(_, initializers, info) => {
                for (_, expr) in initializers {
                    expr.apply(s)?;
                }
                info.apply(s)
            }
            Expression::AccessField(_, _, instance, _, info, type_) => {
                instance.apply(s)?;
                info.apply(s)?;
                type_.apply(s)
            }
            Expression::List(_, elements, element_type) => {
                for element in elements {
                    element.apply(s)?;
                }
                element_type.apply(s)
            }
        }
    }
}

impl ApplySubstitution for Statement {
    fn apply(&mut self, s: &Substitution) -> Result<(), SubstitutionError> {
        match self {
            Statement::Expression(e) => e.apply(s),
            Statement::DefineVariable(_, _, e, type_) => {
                e.apply(s)?;
                type_.apply(s)
            }
            Statement::DefineFunction(_, _, _, parameters, body, return_type) => {
                for (_, _, parameter_type) in parameters {
                    parameter_type.apply(s)?;
                }
                if let Some(body) = body {
                    body.apply(s)?;
                }
                return_type.apply(s)
            }
            Statement::DefineDimension(_, _) => Ok(()),
            Statement::DefineBaseUnit(_, _, type_) => type_.apply(s),
            Statement::DefineDerivedUnit(_, e, _, type_) => {
                e.apply(s)?;
                type_.apply(s)
            }
            Statement::ProcedureCall(_, args) => {
                for arg in args {
                    arg.apply(s)?;
                }
                Ok(())
            }
            Statement::DefineStruct(info) => {
                info.apply(s)?;

                Ok(())
            }
        }
    }
}
