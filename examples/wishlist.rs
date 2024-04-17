use diesel::{Queryable, Selectable};
use rdiesel::{select_list, Expr};

mod schema {
    diesel::table! {
        wishes (id) {
            id -> Integer,
            title -> Text,
            price -> Integer,
            body -> Text,
            private -> Bool,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::wishes)]
pub struct Wish {
    pub id: i32,
    pub title: String,
    pub price: i32,
    pub body: String,
    pub private: bool,
}

impl Expr<Wish, i32> for schema::wishes::price {}

pub fn expensive_wishes(conn: &mut diesel::pg::PgConnection) -> Vec<Wish> {
    use schema::wishes::dsl::*;
    select_list(conn, price.gt(1000)).unwrap()
}

fn main() {}
