#![allow(dead_code)]

use diesel::QueryResult;

pub trait Expr<R, V>: Sized {
    fn eq<E>(self, rhs: E) -> Eq<V, Self, E> {
        Eq {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn lt<E>(self, rhs: E) -> Lt<V, Self, E> {
        Lt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn gt<E>(self, rhs: E) -> Gt<V, Self, E> {
        Gt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn eq_any(self, rhs: Vec<V>) -> EqAny<V, Self> {
        EqAny { lhs: self, rhs }
    }

    fn and<E>(self, rhs: E) -> And<Self, E>
    where
        Self: Expr<R, bool>,
        E: Expr<R, bool>,
    {
        And { lhs: self, rhs }
    }

    fn or<E>(self, rhs: E) -> Or<Self, E>
    where
        Self: Expr<R, bool>,
        E: Expr<R, bool>,
    {
        Or { lhs: self, rhs }
    }
}

pub struct And<E1, E2> {
    lhs: E1,
    rhs: E2,
}

impl<R, E1, E2> Expr<R, bool> for And<E1, E2>
where
    E1: Expr<R, bool>,
    E2: Expr<R, bool>,
{
}

pub struct Or<E1, E2> {
    lhs: E1,
    rhs: E2,
}

impl<R, E1, E2> Expr<R, bool> for Or<E1, E2>
where
    E1: Expr<R, bool>,
    E2: Expr<R, bool>,
{
}
pub struct Eq<V, E1, E2> {
    lhs: E1,
    rhs: E2,
    _val: std::marker::PhantomData<V>,
}

impl<R, E1, E2, V> Expr<R, bool> for Eq<V, E1, E2>
where
    E1: Expr<R, V>,
    E2: Expr<R, V>,
{
}

pub struct Gt<V, E1, E2> {
    lhs: E1,
    rhs: E2,
    _val: std::marker::PhantomData<V>,
}

impl<R, E1, E2, V> Expr<R, bool> for Gt<V, E1, E2>
where
    E1: Expr<R, V>,
    E2: Expr<R, V>,
{
}

pub struct Lt<V, E1, E2> {
    lhs: E1,
    rhs: E2,
    _val: std::marker::PhantomData<V>,
}

impl<R, E1, E2, V> Expr<R, bool> for Lt<V, E1, E2>
where
    E1: Expr<R, V>,
    E2: Expr<R, V>,
{
}

pub struct EqAny<V, E> {
    lhs: E,
    rhs: Vec<V>,
}

impl<R, E, V> Expr<R, bool> for EqAny<V, E> where E: Expr<R, V> {}

impl<R> Expr<R, i32> for i32 {}

impl<R> Expr<R, bool> for bool {}

impl<R> Expr<R, String> for String {}

pub fn select_list<Conn, R, Q>(_conn: &mut Conn, _q: Q) -> QueryResult<Vec<R>>
where
    Q: Expr<R, bool>,
{
    todo!()
}

pub fn select_first<R, Q>(_q: Q) -> Option<R>
where
    Q: Expr<R, bool>,
{
    todo!()
}
