use diesel::{associations::Identifiable, Queryable, Selectable};
use rdiesel::{select_list, update_where, Expr, Field};

#[flux_rs::trusted]
mod schema {
    diesel::table! {
        wishes (id) {
            id -> Integer,
            owner -> Integer,
            title -> Text,
            price -> Integer,
            body -> Text,
            access_level -> Text,
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

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::wishes)]
pub struct Wish {
    pub id: i32,
    pub owner: i32,
    pub title: String,
    pub price: i32,
    pub body: String,
    pub access_level: String,
}

impl Field<Wish, i32> for schema::wishes::price {}
impl Expr<Wish, i32> for schema::wishes::price {}

impl Field<Wish, String> for schema::wishes::access_level {}
impl Expr<Wish, String> for schema::wishes::access_level {}

impl Field<Wish, i32> for schema::wishes::owner {}
impl Expr<Wish, i32> for schema::wishes::owner {}

pub fn test1(conn: &mut diesel::pg::PgConnection) -> Vec<Wish> {
    use schema::wishes::dsl::*;
    select_list(conn, price.gt(1000)).unwrap()
}

pub fn test2(conn: &mut diesel::pg::PgConnection, owners: Vec<i32>) -> Vec<Wish> {
    use schema::wishes::dsl::*;

    select_list(
        conn,
        access_level
            .eq("public".to_string())
            .or(owner.eq_any(owners)),
    )
    .unwrap()
}

pub fn test3(conn: &mut diesel::pg::PgConnection) {
    use schema::wishes::dsl::*;

    // UPDATE wishes
    // SET access_level = "private", price = 0
    // WHERE price > 1000
    let _ = update_where(
        conn,
        price.gt(1000),
        (access_level.assign("private".to_string()), price.assign(0)),
    );
}

fn main() {}
