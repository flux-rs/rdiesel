use diesel::{
    expression::AsExpression,
    sql_types::{Bool, SingleValue, SqlType},
    AppearsOnTable, AsChangeset, BoolExpressionMethods as _, Column, Expression,
    ExpressionMethods as _,
};

use crate::{And, Assign, Changeset, Eq, EqAny, Gt, Lt, Or};

pub trait ToDiesel {
    type DieselType;

    fn to_diesel(self) -> <Self as ToDiesel>::DieselType;
}

#[flux_rs::ignore]
const _: () = {
    impl<V, A, B> ToDiesel for Gt<V, A, B>
    where
        A: Expression,
        A::SqlType: SqlType + SingleValue,
        B: AsExpression<A::SqlType>,
    {
        type DieselType = diesel::dsl::Gt<A, B>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.gt(self.rhs)
        }
    }

    impl<V, A, B> ToDiesel for Lt<V, A, B>
    where
        A: Expression,
        A::SqlType: SqlType + SingleValue,
        B: AsExpression<A::SqlType>,
    {
        type DieselType = diesel::dsl::Lt<A, B>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.lt(self.rhs)
        }
    }

    impl<V, A, B> ToDiesel for Eq<V, A, B>
    where
        A: Expression,
        A::SqlType: SqlType + SingleValue,
        B: AsExpression<A::SqlType>,
    {
        type DieselType = diesel::dsl::Eq<A, B>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.eq(self.rhs)
        }
    }

    impl<A, B> ToDiesel for And<A, B>
    where
        A: ToDiesel,
        B: ToDiesel,
        A::DieselType: Expression<SqlType = Bool>,
        B::DieselType: Expression<SqlType = Bool>,
    {
        type DieselType = diesel::dsl::And<A::DieselType, B::DieselType>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.to_diesel().and(self.rhs.to_diesel())
        }
    }

    impl<A, B> ToDiesel for Or<A, B>
    where
        A: ToDiesel,
        B: ToDiesel,
        A::DieselType: Expression<SqlType = Bool>,
        B::DieselType: Expression<SqlType = Bool>,
    {
        type DieselType = diesel::dsl::Or<A::DieselType, B::DieselType>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.to_diesel().or(self.rhs.to_diesel())
        }
    }

    impl<V, T> ToDiesel for EqAny<V, T>
    where
        T: Expression,
        T::SqlType: SqlType + SingleValue,
        V: AsExpression<T::SqlType>,
    {
        type DieselType = diesel::dsl::EqAny<T, Vec<V>>;

        fn to_diesel(self) -> Self::DieselType {
            self.lhs.eq_any(self.rhs)
        }
    }

    // impl<Target, Expr> ToDiesel for Assign<Target, Expr>
    // where
    //     Target: Column,
    //     Target::SqlType: SqlType + SingleValue,
    //     Expr: AsExpression<Target::SqlType>,
    // {
    //     type DieselType = diesel::dsl::Eq<Target, Expr>;

    //     fn to_diesel(self) -> Self::DieselType {
    //         diesel::ExpressionMethods::eq(self.target, self.expr)
    //     }
    // }

    impl<A, B> AsChangeset for Assign<A, B>
    where
        A: Column,
        A::SqlType: SqlType + SingleValue,
        B: AsExpression<A::SqlType>,
        <B as AsExpression<A::SqlType>>::Expression: AppearsOnTable<A::Table>,
    {
        type Target = A::Table;

        type Changeset = <diesel::dsl::Eq<A, B> as AsChangeset>::Changeset;

        fn as_changeset(self) -> Self::Changeset {
            diesel::ExpressionMethods::eq(self.target, self.expr).as_changeset()
        }
    }
};
