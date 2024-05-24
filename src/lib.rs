use diesel::QueryResult;
mod bridge;

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

pub struct Assign<Target, Expr> {
    target: Target,
    expr: Expr,
}

pub trait Field<R, V>: Expr<R, V> {
    fn assign(self, v: V) -> Assign<Self, V> {
        Assign {
            target: self,
            expr: v,
        }
    }
}

pub trait Changeset<R> {}

impl<R, F, V> Changeset<R> for Assign<F, V> {}

impl<R, T0, T1> Changeset<R> for (T0, T1)
where
    T0: Changeset<R>,
    T1: Changeset<R>,
{
}

impl<R, T0, T1, T2> Changeset<R> for (T0, T1, T2)
where
    T0: Changeset<R>,
    T1: Changeset<R>,
    T2: Changeset<R>,
{
}

impl<R, T0, T1, T2, T3> Changeset<R> for (T0, T1, T2, T3)
where
    T0: Changeset<R>,
    T1: Changeset<R>,
    T2: Changeset<R>,
    T3: Changeset<R>,
{
}

impl<R, T0, T1, T2, T3, T4> Changeset<R> for (T0, T1, T2, T3, T4)
where
    T0: Changeset<R>,
    T1: Changeset<R>,
    T2: Changeset<R>,
    T3: Changeset<R>,
    T4: Changeset<R>,
{
}

#[flux_rs::trusted]
pub fn select_list<'query, Conn, R, Q>(conn: &mut Conn, q: Q) -> QueryResult<Vec<R>>
where
    Q: Expr<R, bool>,
    R: bridge::SelectList<'query, Conn, Q>,
{
    R::select_list(conn, q)
}

#[flux_rs::trusted]
pub fn select_first<'query, Conn, R, Q>(conn: &mut Conn, q: Q) -> QueryResult<Option<R>>
where
    Q: Expr<R, bool>,
    R: bridge::SelectFirst<'query, Conn, Q>,
{
    R::select_first(conn, q)
}

#[flux_rs::trusted]
pub fn update_where<Conn, R, Q, C>(conn: &mut Conn, q: Q, v: C) -> QueryResult<usize>
where
    Q: Expr<R, bool>,
    R: bridge::UpdateWhere<Conn, Q, C>,
{
    R::update_where(conn, q, v)
}
