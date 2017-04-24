use futures::future;
use futures::{IntoFuture, Stream};
use futures::future::{LoopFn, BoxFuture, Future, empty};
use hyper;
use hyper::{Body, Uri, StatusCode};
use hyper::client::{Client, HttpConnector, Response};
use hyper::header::{Location};
use mktemp::Temp;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter;
use std::collections::HashMap;
use tokio_core::reactor::{Handle, Remote};
use tera::Tera;

use papers::DocumentSpec;
use error::Error;

enum GetResult {
    Ok(Response),
    Redirect(Uri),
}

pub struct Workspace {
    dir: Temp,
    document_spec: DocumentSpec,
    handle: Handle,
}

/// We ignore file names that end with a slash for now, and always determine the file name from the
/// Uri
/// TODO: investigate Content-Disposition: attachment
fn extract_file_name_from_uri(uri: &Uri) -> Option<String> {
    uri.path().split('/').last().map(|name| name.to_string())
}

fn determine_get_result(res: Response) -> Result<GetResult, Error> {
    match res.status() {
        StatusCode::TemporaryRedirect | StatusCode::PermanentRedirect => {
            match res.headers().get::<Location>() {
                Some(location) => Ok(GetResult::Redirect(location.parse()?)),
                // future::Loop::Continue(self.download_file(location.parse().unwrap())),
                None => Err(Error::UnprocessableEntity),
            }
        },
        StatusCode::Ok => Ok(GetResult::Ok(res)),
        // future::ok(future::Loop::Break(Ok(None)))
        //Ok(future::Loop::Break(Some(response.body()))),
        // }).boxed()
        _ => unreachable!(),
    }
}

fn download_file(handle: &Handle, uri: Uri) -> Box<Future<Item=Vec<u8>, Error=Error>>
{
    // loop_fn is for tail-recursive futures. See:
    // https://docs.rs/futures/0.1.9/futures/future/fn.loop_fn.html
    let client = Client::new(handle);
    Box::new(future::loop_fn(uri, move |uri| {
        client.get(uri)
            .map_err(Error::from)
            .and_then(|res| {
                match determine_get_result(res) {
                    Ok(GetResult::Redirect(redirect_uri)) => {
                        Ok(future::Loop::Continue(redirect_uri))
                    },
                    Ok(GetResult::Ok(res)) => Ok(future::Loop::Break(res.body())),
                    Err(err) => Err(err),
                }
            })
    }).and_then(|body| {
        body.fold(Vec::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            future::ok::<_, hyper::Error>(acc)
        }).map_err(Error::from)
    }))
}

/// Since mktemp::Temp implements Drop by deleting the directory, we don't need to worry about
/// leaving files or directories behind.
impl Workspace {
    pub fn new(remote: Remote, document_spec: DocumentSpec) -> Result<Workspace, io::Error> {
        Ok(Workspace {
            dir: Temp::new_dir()?,
            document_spec,
            handle: remote.handle().unwrap(),
        })
    }

    pub fn execute(self) {

        // self.download_files()
        //     .and_then(|(workspace, files)| unimplemented!());
        self.handle.spawn(empty())
    }

    fn download_template(self) -> Box<Future<Item=(Workspace, String), Error=Error>> {
        let (handle, uri, variables) = {
            (
                self.handle.clone(),
                self.document_spec.template_url.0.clone(),
                self.document_spec.variables.clone().unwrap_or(HashMap::new()),
            )
        };
        Box::new(
            download_file(&handle, uri)
                .and_then(|bytes| {
                    future::result(::std::string::String::from_utf8(bytes))
                        .map_err(Error::from)
                }).and_then(move |template_string| {
                    Tera::one_off(&template_string, &variables, false)
                        .map_err(Error::from)
                }).map(|latex_doc| (self, latex_doc))
        )
    }

    fn download_assets(self) -> BoxFuture<(Workspace, Vec<u8>), io::Error> {
        // self.download_file(self.document_spec.template_url).then(
    // if let Some(file_name) = extract_file_name_from_uri(&uri) {
        // self.document_spec.template_url
        // future::join_all(
        //     self.document_spec.assets_urls
        //         .unwrap_or(Vec::new())
        //         .iter()
        //         .map(|uri| self.download_file(uri.0)));
        // // save each file as uri.path().split('/').last()

        unimplemented!()
    }

    fn generate_latex(files: Vec<String>) -> BoxFuture<(), io::Error> {
        unimplemented!()
    }

    fn run_latex<T: Read>() -> BoxFuture<BufReader<T>, io::Error> {
        // tokio_process spawn, check exit code, and then open the file, return an async reader to
        // that file
        unimplemented!()
    }

    fn post_generated_pdf() -> BoxFuture<Response, hyper::Error> {
        unimplemented!()
    }
}
