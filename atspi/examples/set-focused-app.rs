use atspi::events::object::StateChangedEvent;
use atspi::events::ObjectEvents;
use atspi_connection::set_session_accessibility;
use atspi_proxies::proxy_ext::ProxyExt;
use std::error::Error;
use tokio_stream::StreamExt;
use atspi::proxy::accessible::ObjectRefExt;
use futures::future::try_join_all;
use std::collections::{hash_map, HashMap};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let atspi = atspi::AccessibilityConnection::new().await?;
	let conn = atspi.connection();
	set_session_accessibility(true).await?;

    let mut desired_app = String::new();
    println!("Enter the name of the application you want to focus: ");
    std::io::stdin().read_line(&mut desired_app).unwrap();

	let root = atspi.root_accessible_on_registry().await?;

	for apps in root.get_children().await?.iter() {
		let proxy = apps.clone().into_accessible_proxy(conn).await?;
		let natural_name = proxy.name().await?.to_lowercase().trim().to_string();
        if natural_name.contains(&desired_app.to_lowercase().trim().to_string()) {
			let actions = proxy.proxies().await?.action().await?;
			let action_list = actions.get_actions().await?;
			for i in 0..action_list.len() {
				println!("{}: {:#?}", i, action_list[i]);
			}
			actions.do_action(1).await?;
        } 
	}
	Ok(())
}
