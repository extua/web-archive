pub use error::Error;
pub use page_archive::PageArchive;
use parsing::{parse_resource_urls, Resource, ResourceMap, ResourceUrl};
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

pub mod error;
pub mod page_archive;
pub mod parsing;

#[cfg(feature = "blocking")]
pub mod blocking;

pub async fn archive<U>(url: U) -> Result<PageArchive, Error>
where
    U: TryInto<Url>,
    <U as TryInto<Url>>::Error: Display,
{
    let url: Url = url
        .try_into()
        .map_err(|e| Error::ParseError(format!("{}", e)))?;

    // Initialise client
    let client = reqwest::Client::new();

    // Fetch the page contents
    let content = client.get(url).send().await?.text().await?;

    // Determine the resources that the page needs
    let resource_urls = parse_resource_urls(&content)?;

    // Download them
    let mut resource_map = ResourceMap::new();
    for resource_url in resource_urls {
        use ResourceUrl::*;
        match resource_url {
            Image(u) => {
                let content =
                    client.get(u.clone()).send().await?.bytes().await?;
                resource_map.insert(u, Resource::Image(content));
            }
            Css(u) => {
                let content =
                    client.get(u.clone()).send().await?.text().await?;
                resource_map.insert(u, Resource::Css(content));
            }
            Javascript(u) => {
                let content =
                    client.get(u.clone()).send().await?.text().await?;
                resource_map.insert(u, Resource::Javascript(content));
            }
        }
    }

    Ok(PageArchive {
        content,
        resource_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn parse_invalid_url_async() {
        let u = "this~is~not~a~url";

        let res = block_on(archive(u));
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}

#[cfg(all(test, feature = "blocking"))]
mod tests_blocking {
    use super::*;

    #[test]
    fn parse_invalid_url_async() {
        let u = "this~is~not~a~url";

        let res = blocking::archive(u);
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}