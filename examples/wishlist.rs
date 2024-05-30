use diesel::{associations::Identifiable, Queryable, Selectable};
use flux_rs::*;
use rdiesel::{select_list, update_where, AuthProvider, Context, Expr, Field};

#[trusted]
mod schema {
    diesel::table! {
        users (id) {
            id -> Integer,
        }
    }
    diesel::table! {
        wishes (id) {
            id -> Integer,
            owner -> Integer,
            title -> Text,
            price -> Integer,
            body -> Text,
            access_level -> Integer,
        }
    }

    diesel::table! {
        friendships (id) {
            id -> Int4,
            user1 -> Varchar,
            user2 -> Varchar,
            friend_status -> Varchar,
        }
    }
}

#[derive(Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::users)]
#[refined_by(id: int)]
pub struct User {
    #[field(i32[id])]
    pub id: i32,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::wishes)]
#[refined_by(id: int, owner: int, price: int, level: int)]
pub struct Wish {
    #[field(i32[id])]
    pub id: i32,
    #[field(i32[owner])]
    pub owner: i32,
    pub title: String,
    #[field(i32[price])]
    pub price: i32,
    pub body: String,
    #[field(i32[level])]
    pub access_level: i32,
}

// Wish.id

#[assoc(fn policy(user: User, wish: Wish) -> bool { false })]
impl Field<Wish, i32, User> for schema::wishes::id {}

#[assoc(fn eval(v: Self, wish: Wish) -> int { wish.price })]
impl Expr<Wish, i32> for schema::wishes::id {}

// Wish.price

#[assoc(fn policy(user: User, wish: Wish) -> bool { user.id == wish.owner })]
impl Field<Wish, i32, User> for schema::wishes::price {}

#[assoc(fn eval(v: Self, wish: Wish) -> int { wish.price })]
impl Expr<Wish, i32> for schema::wishes::price {}

// Wish.access_level

#[assoc(fn policy(user: User, wish: Wish) -> bool { user.id == wish.owner })]
impl Field<Wish, i32, User> for schema::wishes::access_level {}

#[assoc(fn eval(v: Self, wish: Wish) -> int { wish.level })]
impl Expr<Wish, i32> for schema::wishes::access_level {}

// Wish.owner

#[assoc(fn policy(user: User, wish: Wish) -> bool { false })]
impl Field<Wish, i32, User> for schema::wishes::owner {}

#[assoc(fn eval(v: Self, wish: Wish) -> int { wish.owner })]
impl Expr<Wish, i32> for schema::wishes::owner {}

#[sig(fn(bool[true]))]
fn assert(_: bool) {}

pub fn test1(conn: &mut diesel::pg::PgConnection) -> Vec<Wish> {
    use schema::wishes::dsl::*;
    select_list(conn, price.gt(1000)).unwrap()
}

pub fn test2(conn: &mut diesel::pg::PgConnection, owners: Vec<i32>) -> Vec<Wish> {
    use schema::wishes::dsl::*;

    select_list(conn, access_level.eq(0).or(owner.eq_any(owners))).unwrap()
}

pub fn test3(conn: &mut diesel::pg::PgConnection) {
    use schema::wishes::dsl::*;

    // UPDATE wishes
    // SET access_level = "private", price = 0
    // WHERE price > 1000
    let _ = update_where(
        conn,
        price.gt(1000),
        (access_level.assign(1), price.assign(0)),
    );
}

pub fn test4(conn: &mut diesel::pg::PgConnection, owner_id: i32) {
    use schema::wishes::dsl::*;

    let wish_list = select_list(conn, owner.eq(owner_id)).unwrap();

    for wish in wish_list {
        assert(wish.owner == owner_id);
    }
}

impl AuthProvider for &User {
    type User = User;

    fn authenticate(&self) -> Option<Self::User> {
        Some((*self).clone())
    }
}

pub fn test5(conn: diesel::pg::PgConnection, user: User) {
    use schema::wishes::dsl::*;

    let mut cx = Context::new(conn, &user);

    let Some(auth_user) = cx.require_auth_user() else {
        return;
    };

    let wish_list = cx.select_list(owner.eq(auth_user.id)).unwrap();

    for wish in wish_list {
        assert(wish.owner == auth_user.id);
    }
}

pub fn test6(conn: diesel::PgConnection, user: User) {
    use schema::wishes::dsl::*;

    let mut cx = Context::new(conn, &user);

    let Some(auth_user) = cx.require_auth_user() else {
        return;
    };

    cx.update_where(owner.eq(auth_user.id), access_level.assign(1))
        .unwrap();
}

fn main() {}
