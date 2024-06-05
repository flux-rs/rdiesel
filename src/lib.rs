use diesel::QueryResult;
use flux_rs::*;
mod bridge;

/// Dummy trait implemented for every type that can be used as a bound to trick Flux into not
/// generating a kvar when instantiating a type parameter.
pub trait NoKvar {}

impl<T> NoKvar for T {}

pub trait ContextImpl {
    type User;
    type Conn;

    fn auth_user(&self) -> Self::User;

    fn conn(self: &mut Self) -> &mut Self::Conn;
}

flux!(

#[opaque]
pub struct Context<T, U>[user: U] {
    _u: std::marker::PhantomData<U>,
    inner: T,
}

#[trusted]
#[generics(U as base)]
impl<T, U> Context<T, U>
where
    T: ContextImpl<User = U>,
    U: NoKvar,
{
    pub fn new(inner: T) -> Self {
        Self {
            _u: std::marker::PhantomData,
            inner,
        }
    }

    pub fn auth_user(self: &Self[@cx]) -> U[cx.user] {
        self.inner.auth_user()
    }

    pub fn select_list<'query, R as base, Q as base>(
        self: &mut Self[@cx],
        q: Q,
    ) -> QueryResult<Vec<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>
    where
        Q: Expr<R, bool>,
        R: bridge::SelectList<'query, T::Conn, Q>,
    {
        R::select_list(self.inner.conn(), q)
    }

    pub fn select_first<'query, R as base, Q as base>(
        self: &mut Self[@cx],
        q: Q,
    ) -> QueryResult<Option<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>
    where
        Q: Expr<R, bool>,
        R: bridge::SelectFirst<'query, T::Conn, Q>,
    {
        R::select_first(self.inner.conn(), q)
    }

    pub fn update_where<R as base, Q as base, C>(self: &mut Self[@cx], q: Q, v: C) -> QueryResult<usize>
    where
        Q: Expr<R, bool>,
        C: Changeset<R, U>,
        R: bridge::UpdateWhere<T::Conn, Q, C>
    requires forall row. <Q as Expr<R, bool>>::eval(q, row) => <C as Changeset<R, U>>::allow_update(cx.user, row)
    {
        R::update_where(self.inner.conn(), q, v)
    }

    pub fn insert<R as base>(self: &mut Self[@cx], v: R) -> QueryResult<usize>
    where
        R: bridge::Insert<T::Conn>
    {
        R::insert(self.inner.conn(), v)
    }
}

#[generics(Self as base, R as base, V as base)]
pub trait Expr<R, V>: Sized
where
    R: NoKvar,
    V: NoKvar,
{
    reft eval(expr: Self, row: R) -> V;

    fn eq<T as base>(self: Self, rhs: T) -> Eq<V, Self, T>[self, rhs]
    where
        T: NoKvar
    {
        Eq {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn lt<T as base>(self: Self, rhs: T) -> Lt<V, Self, T>[self, rhs]
    where
        T: NoKvar
    {
        Lt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn gt<T as base>(self: Self, rhs: T) -> Gt<V, Self, T>[self, rhs]
    where
        T: NoKvar
    {
        Gt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn eq_any(self: Self, rhs: Vec<V>) -> EqAny<V, Self> {
        EqAny { lhs: self, rhs }
    }

    fn and<T as base>(self: Self, rhs: T) -> And<Self, T>[self, rhs]
    where
        Self: Expr<R, bool>,
        T: Expr<R, bool>,
    {
        And { lhs: self, rhs }
    }

    fn or<T as base>(self: Self, rhs: T) -> Or<Self, T>[self, rhs]
    where
        Self: Expr<R, bool>,
        T: Expr<R, bool>,
    {
        Or { lhs: self, rhs }
    }
}

#[trusted]
#[generics(R as base, U as base)]
pub trait Field<R, V, U>: Sized {
    reft allow_update(user: U, row: R) -> bool;

    fn assign(self: Self, v: V) -> Assign<Self, V> {
        Assign {
            field: self,
            val: v,
        }
    }
}


#[generics(R as base, U as base)]
pub trait Changeset<R, U> {
    reft allow_update(user: U, row: R) -> bool;
}

#[generics(R as base, U as base)]
impl<F, V, R, U> Changeset<R, U> for Assign<F, V> where F: Field<R, V, U> {
    reft allow_update(user: U, row: R) -> bool {
        <F as Field<R, V, U>>::allow_update(user, row)
    }
}

#[generics(R as base, U as base)]
impl<A, B, R, U> Changeset<R, U> for (A, B)
where
    A: Changeset<R, U>,
    B: Changeset<R, U>,
{
    reft allow_update(user: U, row: R) -> bool {
        <A as Changeset<R, U>>::allow_update(user, row) && <B as Changeset<R, U>>::allow_update(user, row)
    }
}

pub struct Assign<F, V> {
    field: F,
    val: V,
}

pub struct And<A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
}


#[generics(R as base, A as base, B as base)]
impl<R, A, B> Expr<R, bool> for And<A, B>
where
    A: Expr<R, bool>,
    B: Expr<R, bool>,
{
    reft eval(expr: And<A, B>, row: R) -> bool {
        <A as Expr<R, bool>>::eval(expr.lhs, row) && <B as Expr<R, bool>>::eval(expr.rhs, row)
    }
}

pub struct Or<A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
}

#[generics(R as base, A as base, B as base)]
impl<R, A, B> Expr<R, bool> for Or<A, B>
where
    A: Expr<R, bool>,
    B: Expr<R, bool>,
{
    reft eval(expr: Or<A, B>, row: R) -> bool {
        <A as Expr<R, bool>>::eval(expr.lhs, row) || <B as Expr<R, bool>>::eval(expr.rhs, row)
    }

}

pub struct Eq<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

#[generics(R as base, A as base, B as base, V as base)]
impl<R, A, B, V> Expr<R, bool> for Eq<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
    reft eval(expr: Eq<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) == <B as Expr<R, V>>::eval(expr.rhs, row)
    }

}

pub struct Gt<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

#[generics(R as base, A as base, B as base, V as base)]
impl<R, A, B, V> Expr<R, bool> for Gt<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
    reft eval(expr: Gt<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) > <B as Expr<R, V>>::eval(expr.rhs, row)
    }

}

pub struct Lt<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

#[generics(R as base, A as base, B as base, V as base)]
impl<R, A, B, V> Expr<R, bool> for Lt<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
    reft eval(expr: Lt<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) < <B as Expr<R, V>>::eval(expr.rhs, row)
    }
}

pub struct EqAny<V, T> {
    lhs: T,
    rhs: Vec<V>,
}

#[generics(R as base, T as base, V as base)]
impl<R, T, V> Expr<R, bool> for EqAny<V, T> where T: Expr<R, V> {
    reft eval(expr: EqAny, row: R) -> bool { true }
}

#[generics(R as base)]
impl<R> Expr<R, i32> for i32 {
    reft eval(val: Self, row: R) -> int { val }
}

#[generics(R as base)]
impl<R> Expr<R, bool> for bool {
    reft eval(val: Self, row: R) -> bool { val }
}

impl<R> Expr<R, String> for String {}

);

#[trusted]
#[sig(fn<R as base, Q as base>(conn: &mut Conn, q: Q) -> QueryResult<Vec<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>)]
pub fn select_list<'query, Conn, R, Q>(conn: &mut Conn, q: Q) -> QueryResult<Vec<R>>
where
    Q: Expr<R, bool>,
    R: bridge::SelectList<'query, Conn, Q>,
{
    R::select_list(conn, q)
}

#[trusted]
#[sig(fn<R as base, Q as base>(conn: &mut Conn, q: Q) -> QueryResult<Option<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>)]
pub fn select_first<'query, Conn, R, Q>(conn: &mut Conn, q: Q) -> QueryResult<Option<R>>
where
    Q: Expr<R, bool>,
    R: bridge::SelectFirst<'query, Conn, Q>,
{
    R::select_first(conn, q)
}

#[trusted]
#[sig(fn<R as base, Q as base>(conn: &mut Conn, q: Q, v: C) -> QueryResult<usize>)]
pub fn update_where<Conn, R, Q, C>(conn: &mut Conn, q: Q, v: C) -> QueryResult<usize>
where
    Q: Expr<R, bool>,
    R: bridge::UpdateWhere<Conn, Q, C>,
{
    R::update_where(conn, q, v)
}
