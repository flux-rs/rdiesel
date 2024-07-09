use flux_rs::*;
use rdiesel::ContextImpl;
use rocket::{
    self,
    http::Status,
    request::{self, FromRequest, Outcome},
    routes, Request,
};
use rocket_dyn_templates::Template;

#[constant]
pub const PUBLIC: i32 = 0;
#[constant]
pub const FRIENDS: i32 = 1;

#[trusted]
mod schema {
    diesel::table! {
        users (id) {
            id -> Integer,
            username -> VarChar,
            password -> VarChar,
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
            user1 -> Integer,
            user2 -> Integer,
            status -> Integer,
        }
    }
}

mod models {
    use crate::schema;
    use diesel::{associations::Identifiable, Insertable, Queryable, Selectable};
    use flux_rs::*;

    flux!(
    #[derive(Clone, Queryable, Selectable, Identifiable)]
    #[diesel(table_name = crate::schema::users)]
    pub struct User[id: int] {
        pub id: i32[id],
        pub username: String,
        pub password: String,
    }

    #[derive(Queryable, Selectable, Identifiable)]
    #[diesel(table_name = crate::schema::wishes)]
    // #[invariant(w == getJust(w.id).level)]
    pub struct Wish[id: int, owner: int, price: int, level: int] {
        pub id: i32[id],
        pub owner: i32[owner],
        pub title: String,
        pub price: i32[price],
        pub body: String,
        pub access_level: i32[level],
    }

    #[derive(Clone, Insertable)]
    #[diesel(table_name = crate::schema::wishes)]
    pub struct NewWish[owner: int, price: int, level: int] {
        pub owner: i32[owner],
        pub title: String,
        pub price: i32[price],
        pub body: String,
        pub access_level: i32[level],
    }

    impl rdiesel::Row<User> for NewWish {
        reft allow_insert(user: User, wish: NewWish) -> bool { user.id == wish.owner }
    }

    // Wish.id

    impl rdiesel::Field<Wish, User> for schema::wishes::id {
        reft allow_update(user: User, wish: Wish) -> bool {
            false
        }
    }

    impl rdiesel::Expr<Wish, i32> for schema::wishes::id {
        reft eval(v: Self, wish: Wish) -> int { wish.price }
    }

    // Wish.price

    impl rdiesel::Field<Wish, User> for schema::wishes::price {
        reft allow_update(user: User, wish: Wish) -> bool { user.id == wish.owner }
    }

    impl rdiesel::Expr<Wish, i32> for schema::wishes::price {
        reft eval(v: Self, wish: Wish) -> int { wish.price }
    }

    // Wish.access_level

    impl rdiesel::Field<Wish, User> for schema::wishes::access_level {
        reft allow_update(user: User, wish: Wish) -> bool { user.id == wish.owner }
    }

    impl rdiesel::Expr<Wish, i32> for schema::wishes::access_level {
        reft eval(v: Self, wish: Wish) -> int { wish.level }
    }

    // Wish.owner

    impl rdiesel::Field<Wish, User> for schema::wishes::owner {
        reft allow_update(user: User, wish: Wish) -> bool { false }
    }

    impl rdiesel::Expr<Wish, i32> for schema::wishes::owner {
        reft eval(v: Self, wish: Wish) -> int { wish.owner }
    }

    // Wish.body

    impl rdiesel::Field<Wish, User> for schema::wishes::body {
        reft allow_update(user: User, wish: Wish) -> bool { user.id == wish.owner }
    }

    #[derive(Queryable, Selectable, Identifiable)]
    #[diesel(table_name = crate::schema::friendships)]
    pub struct Friendship[id: int, user1: int, user2: int, status: int] {
        pub id: i32[id],
        pub user1: i32[user1],
        pub user2: i32[user2],
        pub status: i32[status],
    }

    // Friendship.id

    impl rdiesel::Field<Friendship, User> for schema::friendships::id {
        reft allow_update(user: User, f: Friendship) -> bool { false }
    }

    impl rdiesel::Expr<Friendship, i32> for schema::friendships::id {
        reft eval(v: Self, f: Friendship) -> int { f.id }
    }

    // Friendship.user1

    impl rdiesel::Field<Friendship, User> for schema::friendships::user1 {
        reft allow_update(user: User, f: Friendship) -> bool { false }
    }

    impl rdiesel::Expr<Friendship, i32> for schema::friendships::user1 {
        reft eval(v: Self, f: Friendship) -> int { f.user1 }
    }

    // Friendship.user2

    impl rdiesel::Field<Friendship, User> for schema::friendships::user2 {
        reft allow_update(user: User, f: Friendship) -> bool { false }
    }

    impl rdiesel::Expr<Friendship, i32> for schema::friendships::user2 {
        reft eval(v: Self, f: Friendship) -> int { f.user2 }
    }

    // Friendship.status

    impl rdiesel::Field<Friendship, User> for schema::friendships::status {
        reft allow_update(user: User, f: Friendship) -> bool { false }
    }

    impl rdiesel::Expr<Friendship, i32> for schema::friendships::status {
        reft eval(v: Self, f: Friendship) -> int { f.status }
    }
    );

    impl diesel::associations::HasTable for NewWish {
        type Table = crate::schema::wishes::table;

        fn table() -> Self::Table {
            crate::schema::wishes::table
        }
    }
}

pub struct Session {
    conn: diesel::pg::PgConnection,
    user: models::User,
}

impl Session {
    fn into_context(self) -> Context {
        Context::new(self)
    }
}

type Context = rdiesel::Context<Session, models::User>;

impl ContextImpl for Session {
    type User = models::User;
    type Conn = diesel::pg::PgConnection;

    fn auth_user(&self) -> models::User {
        self.user.clone()
    }

    fn conn(&mut self) -> &mut Self::Conn {
        &mut self.conn
    }
}

pub fn establish_connection() -> diesel::pg::PgConnection {
    todo!()
}

#[flux_rs::ignore]
const _: () = {
    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for Session {
        type Error = ();

        async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
            use schema::users;

            let Some(user_id) = req
                .cookies()
                .get("user_id")
                .and_then(|it| it.value().parse::<i32>().ok())
            else {
                return Outcome::Error((Status::Unauthorized, ()));
            };

            let mut conn = establish_connection();
            let Some(user) = users::table
                .filter(users::id.eq(user_id))
                .first(&mut conn)
                .ok()
            else {
                return Outcome::Error((Status::Unauthorized, ()));
            };
            request::Outcome::Success(Session { conn, user })
        }
    }
};

pub mod services {
    use crate::{
        models::{NewWish, Wish},
        schema::*,
        Session, FRIENDS, PUBLIC,
    };
    use flux_rs::*;
    use rdiesel::{Expr, Field};

    #[sig(fn(bool[true]))]
    fn assert(_: bool) {}

    pub fn update_description(sess: Session, wish_id: i32, new_description: String) {
        let mut cx = sess.into_context();

        let auth_user = cx.auth_user();
        cx.update_where(
            wishes::id.eq(wish_id).and(wishes::owner.eq(auth_user.id)),
            wishes::body.assign(new_description),
        )
        .unwrap();
    }

    #[rocket::get("/user/<user_id>")]
    pub fn user_show(sess: Session, user_id: i32) {
        let mut cx = sess.into_context();

        let auth_user = cx.auth_user();

        let friends = cx
            .select_first(
                friendships::user1
                    .eq(user_id)
                    .and(friendships::user2.eq(auth_user.id)),
            )
            .unwrap()
            .is_some();

        let wishes = if auth_user.id == user_id {
            cx.select_list(wishes::owner.eq(user_id))
        } else if friends {
            cx.select_list(
                wishes::owner.eq(user_id).and(
                    wishes::access_level
                        .eq(PUBLIC)
                        .or(wishes::access_level.eq(FRIENDS)),
                ),
            )
        } else {
            cx.select_list(
                wishes::owner
                    .eq(user_id)
                    .and(wishes::access_level.eq(PUBLIC)),
            )
        };
        // With unwrap verification is way slower
        // let wishes = wishes.unwrap();
        let Ok(wishes) = wishes else {
            return;
        };

        for w in wishes {
            assert(w.owner == user_id);
        }
    }

    pub fn foo(sess: Session) {
        let mut cx = sess.into_context();
        let wishes: Vec<Wish> = cx.select_list(true).unwrap();
    }

    #[rocket::put("/wish")]
    pub fn new_wish(sess: Session) {
        let mut cx = sess.into_context();

        let auth_user = cx.auth_user();

        let wish = NewWish {
            owner: auth_user.id,
            title: "New wish".to_string(),
            price: 100,
            body: "I want this".to_string(),
            access_level: PUBLIC,
        };

        let _ = cx.insert(wish);
    }
}

#[flux_rs::ignore]
fn main() {
    let _ = rocket::async_main(
        rocket::build()
            .mount("/", routes![services::user_show])
            .mount("/", routes![services::new_wish])
            .attach(Template::fairing())
            .launch(),
    );
}
