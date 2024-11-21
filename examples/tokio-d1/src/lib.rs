use worker::*;

use serde::{Deserialize,Serialize};


#[allow(non_snake_case)]
#[derive(Deserialize,Serialize)]
struct Customers {
	CustomerId: String,
	CompanyName: String,
	ContactName: String
}

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let d1 = env.d1("DB")?;
    let statement = d1.prepare("SELECT * FROM Customers WHERE id = ?1");
    let query = statement.bind(&["1".into()])?;
	let result = query.first::<Customers>(None).await?;
	match result {
        Some(customer) => Response::from_json(&customer),
        None => Response::error("Not found", 404),
    }
}
