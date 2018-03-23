use schema::{posts, users};
use uuid::Uuid;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
// use diesel::pg::Pg;
use diesel::helper_types::*;

#[derive(Identifiable, PartialEq, Debug, Queryable, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub privileges: i16,
    pub password: String,
}
impl User {
    pub fn with_name(name: &str) -> FindBy<users::table, users::name, &str> {
        users::table.filter(users::name.eq(name))
    }
}

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub id: Uuid,
    pub name: String,
    pub password: String,
}

impl NewUser {
    pub fn new<S, T>(name: S, password: T) -> Self
        where S: Into<String>,
              T: Into<String>
    {
        NewUser {
            id: Uuid::new_v4(),
            name: name.into(),
            password: password.into(),
        }
    }
}

#[derive(Associations, Identifiable, PartialEq, Debug, Queryable, Serialize)]
#[belongs_to(User, foreign_key="author")]
pub struct Post {
    id: Uuid,
    pub title: String,
    pub text: String,
    pub author: Uuid,
    datetime: NaiveDateTime,
}

#[allow(dead_code)]
impl Post {
    // pub fn from_id(conn: &PgConnection, id: Uuid) -> QueryResult<Post> {
    //     use schema::posts::dsl::*;

    //     posts.find(id).first(conn)
    // }

    // pub fn with_title<'a>(title: &'a str) -> posts::BoxedQuery<'a, Pg> {
    //     posts::table.filter(posts::title.eq(title)).into_boxed()
    // }
    pub fn with_title(title: &str) -> FindBy<posts::table, posts::title, &str> {
        posts::table.filter(posts::title.eq(title))
    }

    pub fn get_uuid(&self) -> Uuid {
        self.id
    }
    pub fn get_datetime(&self) -> NaiveDateTime {
        self.datetime
    }
}

#[derive(Debug, Insertable)]
#[table_name="posts"]
pub struct NewPost {
    id: Uuid,
    pub title: String,
    pub text: String,
    author: Uuid,
}

impl NewPost {
    pub fn new<S, T>(title: S, text: T, author: Uuid) -> Self
        where S: Into<String>,
              T: Into<String>
    {
        NewPost {
            id: Uuid::new_v4(),
            title: title.into(),
            text: text.into(),
            author: author,
        }
    }

    pub fn insert(self, conn: &PgConnection) -> QueryResult<Post> {
        diesel::insert_into(posts::table).values(&self).get_result(conn)
    }
}

// impl Post {
//     pub fn new(title: String, text: String, author: Uuid) -> Self {
//         let datetime: NaiveDateTime = Utc::now().naive_utc();
//         let id: Uuid = Uuid::new_v4();
//         Post { id, title, text, author, datetime }
//     }
// }
