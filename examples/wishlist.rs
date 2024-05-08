use diesel::{associations::Identifiable, Queryable, Selectable};
use rdiesel::{select_list, update_where, Expr};

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

impl Expr<Wish, i32> for schema::wishes::price {}
impl Expr<Wish, String> for schema::wishes::access_level {}
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
    use diesel::RunQueryDsl;
    // use schema::friendships::dsl::*;
    use schema::wishes::dsl::*;

    // update_where(conn, access_level.eq("public".to_string()));
    update_where(
        conn,
        access_level.eq("public".to_string()),
        diesel::ExpressionMethods::eq(access_level, "public".to_string()),
    );
    // diesel::update(diesel::QueryDsl::filter(
    //     wishes,
    //     diesel::ExpressionMethods::eq(access_level, "public".to_string()),
    // ))
    // .set(diesel::ExpressionMethods::eq(
    //     access_level,
    //     "public".to_string(),
    // ))
    // .execute(conn);

    // diesel::update(wishes)
    //     .filter(diesel::ExpressionMethods::eq(
    //         access_level,
    //         "public".to_string(),
    //     ))
    //     .set(diesel::ExpressionMethods::eq(
    //         access_level,
    //         "public".to_string(),
    //     ))
    //     .execute(conn);
}

fn main() {}
