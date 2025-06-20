//! This example prints out the text under the mouse cursor
//!
//! ```sh
//! cargo run --example text-under-mouse
//! ```
//! Authors:
//!    Colton Loftus

use atspi::{MouseEvents};
use atspi_connection::set_session_accessibility;
use atspi_proxies::{
	accessible::{AccessibleProxy, ObjectRefExt},
	proxy_ext::ProxyExt,
};
use futures_lite::stream::StreamExt;
use std::{error::Error, thread::sleep, time::Duration};

async fn get_text_in_app<'a>(
	app: &'a AccessibleProxy<'a>,
	conn: &'a zbus::Connection,
	x: i32,
	y: i32,
) -> Result<String, Box<dyn Error>> {
	Ok(app
		.proxies()
		.await?
		.component()
		.await?
		.get_accessible_at_point(x, y, atspi::CoordType::Screen)
		.await?
		.as_accessible_proxy(conn)
		.await?
		.proxies()
		.await?
		.text()
		.await?
		.get_text(0, i32::MAX)
		.await?)
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let atspi = atspi::AccessibilityConnection::new().await?;
	let conn = atspi.connection();
	set_session_accessibility(true).await?;
	atspi.register_event::<MouseEvents>().await?;

	let apps = atspi.root_accessible_on_registry().await?.get_children().await?;

	let mut events = atspi.event_stream();

	let mut firefox: Option<AccessibleProxy> = None;

	for app in apps.iter() {

		let proxy = app.clone().into_accessible_proxy(conn).await?;
		let natural_name = proxy.name().await?;
		if "Firefox" == natural_name {
			firefox = Some(proxy);
			break;
		}
	}

	let firefox = match firefox {
		Some(firefox) => firefox,
		None => panic!("Firefox not found"),
	};

	while let Some(ev) = events.next().await {

		// doesn't work even if sleep is added to prevent too much throughput
		// sleep(Duration::from_millis(100));

		let ev = match ev {
			Ok(ev) => ev,
			Err(err) => {
				eprintln!("Error: {err}");
				continue;
			}
		};

		let mouse_ev = match ev {
			atspi::Event::Mouse(ev) => ev,
			_ => continue,
		};

		let mouse_abs_ev = match mouse_ev {
			atspi::MouseEvents::Abs(mouse_ev) => mouse_ev,
			_ => continue,

		};

		match get_text_in_app(&firefox, conn, mouse_abs_ev.x, mouse_abs_ev.y).await {
			Ok(text) => println!("{text}"),
			Err(err) => eprintln!("Error geting hovered text in firefox: {err}"),
		}

		// this also fails
		// let text = mouse_abs_ev.item.into_accessible_proxy(conn).await?.proxies().await?.text().await?.get_text(0, i32::MAX).await?;
		// print!("{text}");

	}
	Ok(())
}
