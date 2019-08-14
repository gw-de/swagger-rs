//! Methods for retrieving swagger-related information from an HTTP request.
use hyper::Request;

/// A macro for joining together two or more RequestParsers to create a struct that implements
/// RequestParser with a function parse_operation_id that matches hyper requests against the different
/// RequestParsers in turn until it gets a match (or returns an error if none match)
///
/// The order in which the request parsers are passed to the macro specifies the order in which the request
/// is tested against them. If there is any possibility of two RequestParsers matching the same request
/// this should not be used.
#[macro_export]
macro_rules! request_parser_joiner {
    ($name:ident ,$($T:ty), *) => {
        struct $name;

        impl RequestParser for $name {
            fn parse_operation_id(request: &hyper::Request) -> Result<&'static str, ()> {
                __impl_request_parser_joiner!(request, $($T), *)
            }
        }
    };
}

/// This macro should only be used by the request_parser_joiner macro
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_request_parser_joiner {
    ($argname:expr, $head:ty) => {<$head as RequestParser>::parse_operation_id(&$argname)};
    ($argname:expr, $head:ty, $( $tail:ty), *) => {
        match <$head as RequestParser>::parse_operation_id(&$argname) {
                Ok(s) => Ok(s),
                Err(_) => __impl_request_parser_joiner!($argname, $( $tail), *),
        }
    };
}

/// A trait for retrieving swagger-related information from a request.
///
/// This allows other middlewares to retrieve API-related information from a request that
/// may not have been handled by the autogenerated API code yet.   For example, a statistics
/// tracking service may wish to use this to count requests per-operation.
///
/// The trait is automatically implemented by swagger-codegen.
pub trait RequestParser {
    /// Retrieve the Swagger operation identifier that matches this request.
    ///
    /// Returns `Err(())` if this request does not match any known operation on this API.
    fn parse_operation_id(req: &Request) -> Result<&'static str, ()>;
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use hyper::{Method, Uri};
    use std::str::FromStr;

    struct TestParser1;

    impl RequestParser for TestParser1 {
        fn parse_operation_id(request: &hyper::Request) -> Result<&'static str, ()> {
            match request.uri().path() {
                "/test/t11" => Ok("t11"),
                "/test/t12" => Ok("t12"),
                _ => Err(()),
            }
        }
    }

    struct TestParser2;

    impl RequestParser for TestParser2 {
        fn parse_operation_id(request: &hyper::Request) -> Result<&'static str, ()> {
            match request.uri().path() {
                "/test/t21" => Ok("t21"),
                "/test/t22" => Ok("t22"),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn test_macros() {
        let uri = Uri::from_str(&"https://www.rust-lang.org/test/t11").unwrap();
        let req1: Request = Request::new(Method::Get, uri);

        let uri = Uri::from_str(&"https://www.rust-lang.org/test/t22").unwrap();
        let req2: Request = Request::new(Method::Get, uri);

        let uri = Uri::from_str(&"https://www.rust-lang.org/test/t33").unwrap();
        let req3: Request = Request::new(Method::Get, uri);

        request_parser_joiner!(JoinedReqParser, TestParser1, TestParser2);

        assert_eq!(JoinedReqParser::parse_operation_id(&req1), Ok("t11"));
        assert_eq!(JoinedReqParser::parse_operation_id(&req2), Ok("t22"));
        assert_eq!(JoinedReqParser::parse_operation_id(&req3), Err(()));
    }
}
