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
    use diesel::{associations::Identifiable, Queryable, Selectable};
    use flux_rs::*;
    use rdiesel::{Expr, Field};

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

    #[derive(Queryable, Selectable, Identifiable)]
    #[diesel(table_name = crate::schema::friendships)]
    #[refined_by(id: int, user1: int, user2: int, status: int)]
    pub struct Friendship {
        #[field(i32[id])]
        pub id: i32,
        #[field(i32[user1])]
        pub user1: i32,
        #[field(i32[user2])]
        pub user2: i32,
        #[field(i32[status])]
        pub status: i32,
    }

    // Friendship.id

    #[assoc(fn policy(user: User, f: Friendship) -> bool { false })]
    impl Field<Friendship, i32, User> for schema::friendships::id {}

    #[assoc(fn eval(v: Self, f: Friendship) -> int { f.id })]
    impl Expr<Friendship, i32> for schema::friendships::id {}

    // Friendship.user1

    #[assoc(fn policy(user: User, f: Friendship) -> bool { false })]
    impl Field<Friendship, i32, User> for schema::friendships::user1 {}

    #[assoc(fn eval(v: Self, f: Friendship) -> int { f.user1 })]
    impl Expr<Friendship, i32> for schema::friendships::user1 {}

    // Friendship.user2

    #[assoc(fn policy(user: User, f: Friendship) -> bool { false })]
    impl Field<Friendship, i32, User> for schema::friendships::user2 {}

    #[assoc(fn eval(v: Self, f: Friendship) -> int { f.user2 })]
    impl Expr<Friendship, i32> for schema::friendships::user2 {}

    // Friendship.status

    #[assoc(fn policy(user: User, f: Friendship) -> bool { false })]
    impl Field<Friendship, i32, User> for schema::friendships::status {}

    #[assoc(fn eval(v: Self, f: Friendship) -> int { f.status })]
    impl Expr<Friendship, i32> for schema::friendships::status {}
}

struct Session {
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

mod services {
    use crate::{
        schema::{friendships, wishes},
        Session, FRIENDS, PUBLIC,
    };
    use flux_rs::*;
    use rdiesel::Expr;

    #[sig(fn(bool[true]))]
    fn assert(_: bool) {}

    #[rocket::get("/<user_id>")]
    pub fn user_show(sess: Session, user_id: i32) {
        let mut cx = sess.into_context();

        let auth_user = cx.auth_user();
        let auth_user_id = auth_user.id;

        let friends = cx
            .select_first(
                friendships::user1
                    .eq(user_id)
                    .and(friendships::user2.eq(auth_user_id)),
            )
            .unwrap()
            .is_some();

        let wishes = if auth_user_id == user_id {
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
}

#[flux_rs::ignore]
fn main() {
    let _ = rocket::async_main(
        rocket::build()
            .mount("/", routes![services::user_show])
            .attach(Template::fairing())
            .launch(),
    );
}
