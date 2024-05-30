use diesel::QueryResult;
use flux_rs::*;
mod bridge;

/// Dummy trait with a blanket implementation for every type that can be used as a bound to trick
/// Flux into not generating a kvar when instantiating a type parameter.
pub trait NoKvar {}

impl<T> NoKvar for T {}

pub trait AuthProvider {
    type User;

    fn authenticate(&self) -> Option<Self::User>;
}

flux!(

#[opaque]
pub struct Context<Conn, A, U>[user: U] {
    _u: std::marker::PhantomData<U>,
    conn: Conn,
    auth: A,
}

#[trusted]
#[generics(U as base)]
impl<Conn, A, U> Context<Conn, A, U>
where
    Conn: NoKvar,
    A: NoKvar,
    U: NoKvar,
{
    pub fn new(conn: Conn, auth: A) -> Self {
        Self {
            _u: std::marker::PhantomData,
            conn,
            auth,
        }
    }

    pub fn require_auth_user(self: &Self[@cx]) -> Option<U[cx.user]>
    where
        A: AuthProvider<User = U>,
    {
        self.auth.authenticate()
    }

    pub fn select_list<'query, R as base, Q as base>(
        self: &mut Self[@cx],
        q: Q,
    ) -> QueryResult<Vec<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>
    where
        Q: Expr<R, bool>,
        R: bridge::SelectList<'query, Conn, Q>,
    {
        R::select_list(&mut self.conn, q)
    }

    pub fn select_first<'query, R as base, Q as base>(
        self: &mut Self[@cx],
        q: Q,
    ) -> QueryResult<Option<R{row: <Q as Expr<R, bool>>::eval(q, row)}>>
    where
        Q: Expr<R, bool>,
        R: bridge::SelectFirst<'query, Conn, Q>,
    {
        R::select_first(&mut self.conn, q)
    }

    pub fn update_where<R as base, Q as base, C>(self: &mut Self[@cx], q: Q, v: C) -> QueryResult<usize>
    where
        Q: Expr<R, bool>,
        C: Changeset<R, U>,
        R: bridge::UpdateWhere<Conn, Q, C>
    requires forall row. <Q as Expr<R, bool>>::eval(q, row) => <C as Changeset<R, U>>::policy(cx.user, row)
    {
        R::update_where(&mut self.conn, q, v)
    }
}
);

#[generics(Self as base, R as base, V as base)]
#[assoc(fn eval(expr: Self, row: R) -> V)]
pub trait Expr<R, V>: Sized
where
    R: NoKvar,
    V: NoKvar,
{
    #[sig(fn<T as base>(lhs: Self, rhs: T) -> Eq<V, Self, T>[lhs, rhs])]
    fn eq<T>(self, rhs: T) -> Eq<V, Self, T> {
        Eq {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    #[sig(fn<T as base>(lhs: Self, rhs: T) -> Lt<V, Self, T>[lhs, rhs])]
    fn lt<T>(self, rhs: T) -> Lt<V, Self, T> {
        Lt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    #[sig(fn<T as base>(lhs: Self, rhs: T) -> Gt<V, Self, T>[lhs, rhs])]
    fn gt<T>(self, rhs: T) -> Gt<V, Self, T> {
        Gt {
            _val: std::marker::PhantomData,
            lhs: self,
            rhs,
        }
    }

    fn eq_any(self, rhs: Vec<V>) -> EqAny<V, Self> {
        EqAny { lhs: self, rhs }
    }

    #[sig(fn<T as base>(lhs: Self, rhs: T) -> And<Self, T>[lhs, rhs])]
    fn and<T>(self, rhs: T) -> And<Self, T>
    where
        Self: Expr<R, bool>,
        T: Expr<R, bool>,
    {
        And { lhs: self, rhs }
    }

    #[sig(fn<T as base>(lhs: Self, rhs: T) -> Or<Self, T>[lhs, rhs])]
    fn or<T>(self, rhs: T) -> Or<Self, T>
    where
        Self: Expr<R, bool>,
        T: Expr<R, bool>,
    {
        Or { lhs: self, rhs }
    }
}

#[trusted]
#[generics(R as base, U as base)]
#[assoc(fn policy(user: U, row: R) -> bool)]
pub trait Field<R, V, U>: Sized {
    fn assign(self, v: V) -> Assign<Self, V> {
        Assign {
            field: self,
            val: v,
        }
    }
}

#[generics(R as base, U as base)]
#[assoc(fn policy(user: U, row: R) -> bool)]
pub trait Changeset<R, U> {}

#[generics(R as base, U as base)]
#[assoc(fn policy(user: U, row: R) -> bool { <F as Field<R, V, U>>::policy(user, row) })]
impl<F, V, R, U> Changeset<R, U> for Assign<F, V> where F: Field<R, V, U> {}

#[generics(R as base, U as base)]
#[assoc(fn policy(user: U, row: R) -> bool {
    <A as Changeset<R, U>>::policy(user, row) && <B as Changeset<R, U>>::policy(user, row)
})]
impl<A, B, R, U> Changeset<R, U> for (A, B)
where
    A: Changeset<R, U>,
    B: Changeset<R, U>,
{
}

flux! (

pub struct Assign<F, V> {
    field: F,
    val: V,
}

pub struct And<A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
}

pub struct Or<A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
}

pub struct Eq<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

pub struct Gt<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

pub struct Lt<V, A, B>[lhs: A, rhs: B] {
    lhs: A[lhs],
    rhs: B[rhs],
    _val: std::marker::PhantomData<V>,
}

pub struct EqAny<V, T> {
    lhs: T,
    rhs: Vec<V>,
}

);

#[generics(R as base, A as base, B as base)]
#[assoc(
    fn eval(expr: And<A, B>, row: R) -> bool {
        <A as Expr<R, bool>>::eval(expr.lhs, row) && <B as Expr<R, bool>>::eval(expr.rhs, row)
    }
)]
impl<R, A, B> Expr<R, bool> for And<A, B>
where
    A: Expr<R, bool>,
    B: Expr<R, bool>,
{
}

#[generics(R as base, A as base, B as base)]
#[assoc(
    fn eval(expr: Or<A, B>, row: R) -> bool {
        <A as Expr<R, bool>>::eval(expr.lhs, row) || <B as Expr<R, bool>>::eval(expr.rhs, row)
    }
)]
impl<R, A, B> Expr<R, bool> for Or<A, B>
where
    A: Expr<R, bool>,
    B: Expr<R, bool>,
{
}

#[generics(R as base, A as base, B as base, V as base)]
#[assoc(
    fn eval(expr: Eq<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) == <B as Expr<R, V>>::eval(expr.rhs, row)
    }
)]
impl<R, A, B, V> Expr<R, bool> for Eq<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
}

#[generics(R as base, A as base, B as base, V as base)]
#[assoc(
    fn eval(expr: Gt<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) > <B as Expr<R, V>>::eval(expr.rhs, row)
    }
)]
impl<R, A, B, V> Expr<R, bool> for Gt<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
}

#[generics(R as base, A as base, B as base, V as base)]
#[assoc(
    fn eval(expr: Lt<A, B>, row: R) -> bool {
        <A as Expr<R, V>>::eval(expr.lhs, row) < <B as Expr<R, V>>::eval(expr.rhs, row)
    }
)]
impl<R, A, B, V> Expr<R, bool> for Lt<V, A, B>
where
    A: Expr<R, V>,
    B: Expr<R, V>,
{
}

#[generics(R as base, T as base, V as base)]
#[assoc(fn eval(expr: EqAny, row: R) -> bool { true })]
impl<R, T, V> Expr<R, bool> for EqAny<V, T> where T: Expr<R, V> {}

#[generics(R as base)]
#[assoc(fn eval(val: Self, row: R) -> int { val })]
impl<R> Expr<R, i32> for i32 {}

#[generics(R as base)]
#[assoc(fn eval(val: Self, row: R) -> bool { val })]
impl<R> Expr<R, bool> for bool {}

impl<R> Expr<R, String> for String {}

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
