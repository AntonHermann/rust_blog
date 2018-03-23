extern crate file_server_lib;
extern crate bcrypt;
extern crate diesel;
#[macro_use]
extern crate fake;

use file_server_lib::*;
use file_server_lib::models::*;
use bcrypt::*;
use diesel::prelude::*;

fn generate_user_info(pw: &str) -> NewUser {
    NewUser::new(fake!(Name.name), pw)
}

fn generate_post_info(author: User) -> NewPost {
    let title = fake!(Lorem.sentence(1, 4));
    let text = fake!(Lorem.paragraph(5,5));
    NewPost::new(title, text, author.id)
}

fn main() {
    use schema::posts::dsl::*;
    use schema::users::dsl::*;

    let conn = init_pool().get().unwrap();

    let plain_text_password = "test123";
    let hashed_password = hash(plain_text_password, DEFAULT_COST)
        .expect("error hashing password");

    let del_p_c = diesel::delete(posts)
        .execute(&*conn).expect("error deleting posts");
    eprintln!("deleted {} posts", del_p_c);
    let del_u_c = diesel::delete(users).execute(&*conn)
        .expect("error deleting users");
    eprintln!("deleted {} users", del_u_c);


    let me = NewUser::new("Anton", hashed_password.clone());

    let me_c = diesel::insert_into(users).values(&me)
        .execute(&*conn).expect("error inserting user");
    eprintln!("inserted {} user", me_c);

    let new_user_list: Vec<NewUser> =
        (0..10).map(|_| generate_user_info(&hashed_password)).collect();

    let returned_users = diesel::insert_into(users)
        .values(&new_user_list).get_results::<User>(&*conn)
        .expect("error inserting users");
    eprintln!("inserted {} users", returned_users.len());

    let new_post_list: Vec<NewPost> = returned_users.into_iter()
        .map(generate_post_info).collect();

    let post_c = diesel::insert_into(posts)
        .values(&new_post_list).execute(&*conn)
        .expect("error inserting posts");
    eprintln!("inserted {} posts", post_c);
}
