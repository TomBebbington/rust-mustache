use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::str;
use serialize::Encodable;

use compiler::Compiler;
use data::Data;
use encoder::Encoder;
use encoder;
use error::Error;
use parser::Token;
use parser;
use context::Context;

/// `Template` represents a compiled mustache file.
#[derive(Debug, Clone)]
pub struct Template {
    ctx: Context,
    tokens: Vec<Token>,
    partials: HashMap<String, Vec<Token>>
}

/// Construct a `Template`. This is not part of the impl of Template so it is
/// not exported outside of mustache.
pub fn new(ctx: Context, tokens: Vec<Token>, partials: HashMap<String,
Vec<Token>>) -> Template {
    Template {
        ctx: ctx,
        tokens: tokens,
        partials: partials,
    }
}

impl Template {
    /// Renders the template with the `Encodable` data.
    pub fn render<'a, W: Write, T: Encodable>(
        &self,
        wr: &mut W,
        data: &T
    ) -> Result<(), Error> {
        let data = try!(encoder::encode(data));
        Ok(self.render_data(wr, &data))
    }

    /// Renders the template with the `Data`.
    pub fn render_data<W: Write>(&self, wr: &mut W, data: &Data) {
        let mut render_ctx = RenderContext::new(self);
        let mut stack = vec!(data);

        render_ctx.render(
            wr,
            &mut stack,
            &self.tokens);
    }
}

struct RenderContext<'a> {
    template: &'a Template,
    indent: String,
}

impl<'a> RenderContext<'a> {
    fn new(template: &'a Template) -> RenderContext<'a> {
        RenderContext {
            template: template,
            indent: "".to_string(),
        }
    }

    fn render<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        tokens: &[Token]
    ) {
        for token in tokens.iter() {
            self.render_token(wr, stack, token);
        }
    }

    fn render_token<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        token: &Token
    ) {
        match *token {
            Token::Text(ref value) => {
                self.render_text(wr, &value);
            },
            Token::ETag(ref path, _) => {
                self.render_etag(wr, stack, &path);
            }
            Token::UTag(ref path, _) => {
                self.render_utag(wr, stack, &path);
            }
            Token::Section(ref path, true, ref children, _, _, _, _, _) => {
                self.render_inverted_section(wr, stack, &path, &children);
            }
            Token::Section(ref path, false, ref children, ref otag, _, ref src, _, ref ctag) => {
                self.render_section(
                    wr,
                    stack,
                    path,
                    children,
                    src,
                    otag,
                    ctag)
            }
            Token::Partial(ref name, ref indent, _) => {
                self.render_partial(wr, stack, &name, &indent);
            }
            _ => { panic!() }
        }
    }

    fn render_text<W: Write>(
        &mut self,
        wr: &mut W,
        value: &str
    ) {
        // Indent the lines.
        if self.indent.is_empty() {
            wr.write(value.as_bytes()).unwrap();
        } else {
            let mut pos = 0;
            let len = value.len();

            while pos < len {
                let v = value.slice_from(pos);
                let line = match v.find('\n') {
                    None => {
                        let line = v;
                        pos = len;
                        line
                    }
                    Some(i) => {
                        let line = v.slice_to(i + 1);
                        pos += i + 1;
                        line
                    }
                };

                if line.char_at(0) != '\n' {
                    wr.write(self.indent.as_bytes()).unwrap();
                }

                wr.write(line.as_bytes()).unwrap();
            }
        }
    }

    fn render_etag<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        path: &[String]
    ) {
        let mut bytes = Vec::new();

        self.render_utag(&mut bytes, stack, path);

        let s = str::from_utf8(&bytes).unwrap().to_string();

        for c in s.chars() {
            match c {
                '<'  => { wr.write("&lt;".as_bytes()) }
                '>'  => { wr.write("&gt;".as_bytes()) }
                '&'  => { wr.write("&amp;".as_bytes()) }
                '"'  => { wr.write("&quot;".as_bytes()) }
                '\'' => { wr.write("&#39;".as_bytes()) }
                _    => {
                    let mut text:Vec<u8> = (0..c.len_utf8()).map(|_| 0).collect();
                    c.encode_utf8(&mut text);
                    wr.write(&text)
                }
            }.unwrap();
        }
    }

    fn render_utag<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        path: &[String]
    ) {
        match self.find(path, stack) {
            None => { }
            Some(value) => {
                wr.write(self.indent.as_bytes()).unwrap();

                match *value {
                    Data::Str(ref value) => {
                        wr.write(value.as_bytes()).unwrap();
                    }

                    // etags and utags use the default delimiter.
                    Data::Fun(ref f) => {
                        let tokens = self.render_fun("", "{{", "}}", &**f.borrow());
                        self.render(wr, stack, &tokens);
                    }

                    ref value => { panic!("unexpected value {:?}", value); }
                }
            }
        };
    }

    fn render_inverted_section<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        path: &[String],
        children: &[Token]
    ) {
        match self.find(path, stack) {
            None => { }
            Some(&Data::Bool(false)) => { }
            Some(&Data::Vec(ref xs)) if xs.is_empty() => { }
            Some(_) => { return; }
        }

        self.render(wr, stack, children);
    }

    fn render_section<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        path: &[String],
        children: &[Token],
        src: &str,
        otag: &str,
        ctag: &str
    ) {
        match self.find(path, stack) {
            None => { }
            Some(value) => {
                match *value {
                    Data::Bool(true) => {
                        self.render(wr, stack, children);
                    }
                    Data::Bool(false) => { }
                    Data::Vec(ref vs) => {
                        for v in vs.iter() {
                            stack.push(v);
                            self.render(wr, stack, children);
                            stack.pop();
                        }
                    }
                    Data::Map(_) => {
                        stack.push(value);
                        self.render(wr, stack, children);
                        stack.pop();
                    }
                    Data::Fun(ref f) => {
                        let tokens = self.render_fun(src, otag, ctag, &**f.borrow());
                        self.render(wr, stack, &tokens)
                    }
                    _ => { panic!("unexpected value {:?}", value) }
                }
            }
        }
    }

    fn render_partial<'b, W: Write>(
        &mut self,
        wr: &mut W,
        stack: &mut Vec<&Data>,
        name: &str,
        indent: &str
    ) {
        match self.template.partials.get(name) {
            None => { }
            Some(ref tokens) => {
                let mut indent = format!("{}{}", self.indent, indent);

                mem::swap(&mut self.indent, &mut indent);
                self.render(wr, stack, &tokens);
                mem::swap(&mut self.indent, &mut indent);
            }
        }
    }

    fn render_fun(
        &self,
        src: &str,
        otag: &str,
        ctag: &str,
        f: &Fn(String) -> String
    ) -> Vec<parser::Token> {
        let src = (*f)(src.to_string());

        let compiler = Compiler::new_with(
            self.template.ctx.clone(),
            src.chars(),
            self.template.partials.clone(),
            otag.to_string(),
            ctag.to_string());

        let (tokens, _) = compiler.compile();
        tokens
    }

    fn find<'b, 'c>(&self, path: &[String], stack: &mut Vec<&'c Data>) -> Option<&'c Data> {
        // If we have an empty path, we just want the top value in our stack.
        if path.is_empty() {
            match stack.last() {
                None => { return None; }
                Some(data) => { return Some(*data); }
            }
        }

        // Otherwise, find the stack that has the first part of our path.
        let mut value = None;

        for data in stack.iter().rev() {
            match **data {
                Data::Map(ref m) => {
                    match m.get(&path[0]) {
                        Some(v) => {
                            value = Some(v);
                            break;
                        }
                        None => { }
                    }
                }
                _ => { panic!("expect map: {:?}", path) }
            }
        }

        // Walk the rest of the path to find our final value.
        let mut value = match value {
            Some(value) => value,
            None => { return None; }
        };

        for part in path.slice_from(1).iter() {
            match *value {
                Data::Map(ref m) => {
                    match m.get(part) {
                        Some(v) => { value = v; }
                        None => { return None; }
                    }
                }
                _ => { return None; }
            }
        }

        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::str;
    use std::io::{File, TempDir};
    use std::collections::HashMap;
    use serialize::json;
    use serialize::Encodable;

    use context::Context;
    use data::Data;
    use encoder::Encoder;
    use error::Error;
    use template::Template;

    use super::super::compile_str;

    #[derive(Encodable)]
    struct Name { name: String }

    fn render<'a, 'b, T: Encodable<Encoder<'b>, Error>>(
        template: &str,
        data: &T,
    ) -> Result<String, Error> {
        let template = compile_str(template);

        let mut wr = Vec::new();
        try!(template.render(&mut wr, data));

        Ok(String::from_utf8(wr).unwrap().to_string())
    }

    #[test]
    fn test_render_texts() {
        let ctx = Name { name: "world".to_string() };

        assert_eq!(render("hello world", &ctx), Ok("hello world".to_string()));
        assert_eq!(render("hello {world", &ctx), Ok("hello {world".to_string()));
        assert_eq!(render("hello world}", &ctx), Ok("hello world}".to_string()));
        assert_eq!(render("hello {world}", &ctx), Ok("hello {world}".to_string()));
        assert_eq!(render("hello world}}", &ctx), Ok("hello world}}".to_string()));
    }

    #[test]
    fn test_render_etags() {
        let ctx = Name { name: "world".to_string() };

        assert_eq!(render("hello {{name}}", &ctx), Ok("hello world".to_string()));
    }

    #[test]
    fn test_render_utags() {
        let ctx = Name { name: "world".to_string() };

        assert_eq!(render("hello {{{name}}}", &ctx), Ok("hello world".to_string()));
    }

    fn render_data<'a>(template: &Template, data: &Data) -> String {
        let mut wr = Vec::new();
        template.render_data(&mut wr, data);
        String::from_utf8(wr).unwrap().to_string()
    }

    #[test]
    fn test_render_sections() {
        let ctx = HashMap::new();
        let template = compile_str("0{{#a}}1 {{n}} 3{{/a}}5");

        assert_eq!(render_data(&template, &Data::Map(ctx)), "05".to_string());

        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Data::Vec(Vec::new()));

        assert_eq!(render_data(&template, &Data::Map(ctx)), "05".to_string());

        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Data::Vec(Vec::new()));
        assert_eq!(render_data(&template, &Data::Map(ctx)), "05".to_string());

        let mut ctx0 = HashMap::new();
        let ctx1 = HashMap::new();
        ctx0.insert("a".to_string(), Data::Vec(vec!(Data::Map(ctx1))));

        assert_eq!(render_data(&template, &Data::Map(ctx0)), "01  35".to_string());

        let mut ctx0 = HashMap::new();
        let mut ctx1 = HashMap::new();
        ctx1.insert("n".to_string(), Data::Str("a".to_string()));
        ctx0.insert("a".to_string(), Data::Vec(vec!(Data::Map(ctx1))));
        assert_eq!(render_data(&template, &Data::Map(ctx0)), "01 a 35".to_string());

        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Data::Fun(RefCell::new(|_text| "foo".to_string())));
        assert_eq!(render_data(&template, &Data::Map(ctx)), "0foo5".to_string());
    }

    #[test]
    fn test_render_inverted_sections() {
        let template = compile_str("0{{^a}}1 3{{/a}}5");

        let ctx = HashMap::new();
        assert_eq!(render_data(&template, &Data::Map(ctx)), "01 35".to_string());

        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Data::Vec(vec!()));
        assert_eq!(render_data(&template, &Data::Map(ctx)), "01 35".to_string());

        let mut ctx0 = HashMap::new();
        let ctx1 = HashMap::new();
        ctx0.insert("a".to_string(), Data::Vec(vec!(Data::Map(ctx1))));
        assert_eq!(render_data(&template, &Data::Map(ctx0)), "05".to_string());

        let mut ctx0 = HashMap::new();
        let mut ctx1 = HashMap::new();
        ctx1.insert("n".to_string(), Data::Str("a".to_string()));
        ctx0.insert("a".to_string(), Data::Vec(vec!(Data::Map(ctx1))));
        assert_eq!(render_data(&template, &Data::Map(ctx0)), "05".to_string());
    }

    #[test]
    fn test_render_partial() {
        let template = Context::new(Path::new("src/test-data"))
            .compile_path(Path::new("base"))
            .unwrap();

        let ctx = HashMap::new();
        assert_eq!(render_data(&template, &Data::Map(ctx)), "<h2>Names</h2>\n".to_string());

        let mut ctx = HashMap::new();
        ctx.insert("names".to_string(), Data::Vec(vec!()));
        assert_eq!(render_data(&template, &Data::Map(ctx)), "<h2>Names</h2>\n".to_string());

        let mut ctx0 = HashMap::new();
        let ctx1 = HashMap::new();
        ctx0.insert("names".to_string(), Data::Vec(vec!(Data::Map(ctx1))));
        assert_eq!(
            render_data(&template, &Data::Map(ctx0)),
            "<h2>Names</h2>\n  <strong></strong>\n\n".to_string());

        let mut ctx0 = HashMap::new();
        let mut ctx1 = HashMap::new();
        ctx1.insert("name".to_string(), Data::Str("a".to_string()));
        ctx0.insert("names".to_string(), Data::Vec(vec!(Data::Map(ctx1))));
        assert_eq!(
            render_data(&template, &Data::Map(ctx0)),
            "<h2>Names</h2>\n  <strong>a</strong>\n\n".to_string());

        let mut ctx0 = HashMap::new();
        let mut ctx1 = HashMap::new();
        ctx1.insert("name".to_string(), Data::Str("a".to_string()));
        let mut ctx2 = HashMap::new();
        ctx2.insert("name".to_string(), Data::Str("<b>".to_string()));
        ctx0.insert("names".to_string(), Data::Vec(vec!(Data::Map(ctx1), Data::Map(ctx2))));
        assert_eq!(
            render_data(&template, &Data::Map(ctx0)),
            "<h2>Names</h2>\n  <strong>a</strong>\n\n  <strong>&lt;b&gt;</strong>\n\n".to_string());
    }

    fn parse_spec_tests(src: &str) -> Vec<json::Json> {
        let path = Path::new(src);

        let file_contents = match File::open(&path).read_to_end() {
            Ok(reader) => reader,
            Err(e) => panic!("Could not read file {}", e),
        };

        let s = match str::from_utf8(file_contents.as_slice()){
            Some(str) => str.to_string(),
            None => {panic!("File was not UTF8 encoded");}
        };

        match json::from_str(s.as_slice()) {
            Err(e) => panic!(e.to_string()),
            Ok(json) => {
                match json {
                    json::Json::Object(d) => {
                        let mut d = d;
                        match d.remove("tests") {
                            Some(json::Json::Array(tests)) => tests.into_iter().collect(),
                            _ => panic!("{}: tests key not a list", src),
                        }
                    }
                    _ => panic!("{}: JSON value not a map", src),
                }
            }
        }
    }

    fn write_partials(tmpdir: &Path, value: &json::Json) {
        match value {
            &json::Json::Object(ref d) => {
                for (key, value) in d.iter() {
                    match value {
                        &json::Json::String(ref s) => {
                            let mut path = tmpdir.clone();
                            path.push(*key + ".mustache");
                            File::create(&path).write(s.as_bytes()).unwrap();
                        }
                        _ => panic!(),
                    }
                }
            },
            _ => panic!(),
        }
    }

    fn run_test(test: json::Object, data: Data) {
        let template = match test.get("template") {
            Some(&json::Json::String(ref s)) => s.clone(),
            _ => panic!(),
        };

        let expected = match test.get("expected") {
            Some(&json::Json::String(ref s)) => s.clone(),
            _ => panic!(),
        };

        // Make a temporary dir where we'll store our partials. This is to
        // avoid a race on filenames.
        let tmpdir = match TempDir::new("") {
            Ok(tmpdir) => tmpdir,
            Err(_) => panic!(),
        };

        match test.get("partials") {
            Some(value) => write_partials(tmpdir.path(), value),
            None => {},
        }

        let ctx = Context::new(tmpdir.path().clone());
        let template = ctx.compile(template.as_slice().chars());
        let result = render_data(&template, &data);

        if result != expected {
            prisizeln!("desc:     {}", test.get("desc").unwrap().to_string());
            prisizeln!("context:  {}", test.get("data").unwrap().to_string());
            prisizeln!("=>");
            prisizeln!("template: {}", template);
            prisizeln!("expected: {}", expected);
            prisizeln!("actual:   {}", result);
            prisizeln!("");
        }
        assert_eq!(result, expected);
    }

    fn run_tests(spec: &str) {
        for json in parse_spec_tests(spec).into_iter() {
            let test = match json {
                json::Json::Object(m) => m,
                _ => panic!(),
            };

            let data = match test.get("data") {
                Some(data) => data.clone(),
                None => panic!(),
            };

            let mut encoder = Encoder::new();
            data.encode(&mut encoder).unwrap();
            assert_eq!(encoder.data.len(), 1);

            run_test(test, encoder.data.pop().unwrap());
        }
    }

    #[test]
    fn test_spec_comments() {
        run_tests("spec/specs/comments.json");
    }

    #[test]
    fn test_spec_delimiters() {
        run_tests("spec/specs/delimiters.json");
    }

    #[test]
    fn test_spec_isizeerpolation() {
        run_tests("spec/specs/isizeerpolation.json");
    }

    #[test]
    fn test_spec_inverted() {
        run_tests("spec/specs/inverted.json");
    }

    #[test]
    fn test_spec_partials() {
        run_tests("spec/specs/partials.json");
    }

    #[test]
    fn test_spec_sections() {
        run_tests("spec/specs/sections.json");
    }

    #[test]
    fn test_spec_lambdas() {
        for json in parse_spec_tests("spec/specs/~lambdas.json").into_iter() {
            let mut test = match json {
                json::Json::Object(m) => m,
                value => { panic!("{}", value) }
            };

            let s = match test.remove("name") {
                Some(json::Json::String(s)) => s,
                value => { panic!("{}", value) }
            };

            // Replace the lambda with rust code.
            let data = match test.remove("data") {
                Some(data) => data,
                None => panic!(),
            };

            let mut encoder = Encoder::new();
            data.encode(&mut encoder).unwrap();

            let mut ctx = match encoder.data.pop().unwrap() {
                Data::Map(ctx) => ctx,
                _ => panic!(),
            };

            // needed for the closure test.
            let mut calls = 0u;

            let f = match s.as_slice() {
                "Interpolation" => {
                    |_text| { "world".to_string() }
                }
                "Interpolation - Expansion" => {
                    |_text| { "{{planet}}".to_string() }
                }
                "Interpolation - Alternate Delimiters" => {
                    |_text| { "|planet| => {{planet}}".to_string() }
                }
                "Interpolation - Multiple Calls" => {
                    |_text| {
                        calls += 1;
                        calls.to_string()
                    }
                }
                "Escaping" => {
                    |_text| { ">".to_string() }
                }
                "Section" => {
                    |text: String| {
                        if text.as_slice() == "{{x}}" {
                            "yes".to_string()
                        } else {
                            "no".to_string()
                        }
                    }
                }
                "Section - Expansion" => {
                    |text: String| { text + "{{planet}}" + text }
                }
                "Section - Alternate Delimiters" => {
                    |text: String| { text + "{{planet}} => |planet|" + text }
                }
                "Section - Multiple Calls" => {
                    |text: String| { "__".to_string() + text + "__" }
                }
                "Inverted Section" => {
                    |_text| { "".to_string() }
                }

                value => { panic!("{}", value) }
            };

            ctx.insert("lambda".to_string(), Data::Fun(RefCell::new(f)));

            run_test(test, Data::Map(ctx));
        }
    }
}
