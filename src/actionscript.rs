use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Type {
    Object, // o, _o -> prefered when type is unknown
    Array, // a, _a
    Boolean, // b, _b
    String, // s, _s
    Number, // n, _n
}

#[derive(Serialize, Deserialize)]
pub struct Class {
    name: String,
    extends: String,
    implements: String,
    members: Vec<Member>,
    functions: Vec<Function>,
}

#[derive(Serialize, Deserialize)]
pub struct Member {
    is_static: bool,
    name: String,
    var_type: Type,
    value: String,
}
#[derive(Serialize, Deserialize)]
pub struct Arg {
    name: String,
    var_type: Type,
    value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Function {
    is_static: bool,
    name: String,
    args: Vec<Arg>,
    lines: Vec<String>,
}

pub struct Actionscript {
    buffer: Vec<u8>,
}

impl Actionscript {
    #[allow(dead_code)]
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let buffer: Vec<u8> = std::fs::read(path)?;

        Ok(Actionscript {
            buffer,
        })
    }

    #[allow(dead_code)]
    pub fn from_buffer(buffer: &[u8]) -> Self {
        Actionscript {
            buffer: buffer.to_vec(),
        }
    }

    #[allow(dead_code)]
    pub fn from_vector(vec: Vec<u8>) -> Self {
        Actionscript {
            buffer: vec,
        }
    }

    #[allow(dead_code)]
    pub fn to_json(&self, path: &str) -> std::io::Result<()> {
        let object_class: Class = self.interpret();

        if let Ok(json) = serde_json::to_string_pretty(&object_class) {
            std::fs::write(path, json)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn to_object(&self) -> Class {
        self.interpret()
    }
}

// TODO: refactore what I can so no multiple functions who looks the same and put the analysis functions into an Interpreter Trait for easier future more language supp and not just actionscript
impl Actionscript {
    fn interpret(&self) -> Class {
        let (as_class_name, offset) = self.find_unique_type(b"class");
        // if as_class_name == "" -> check singleton instead
        let (as_class_extends, _) = self.find_unique_type_offset(b"extends", offset);
        let (as_class_implements, _) = self.find_unique_type_offset(b"implements", offset);
        let (as_class_functions, as_class_members) = self.parse(offset);

        Class {
            name: as_class_name,
            extends: as_class_extends,
            implements: as_class_implements,
            members: as_class_members,
            functions: as_class_functions,
        }
    }

    fn parse(&self, buffer_offset: usize) -> (Vec<Function>, Vec<Member>) {
        let mut functions_vec: Vec<Function> = Vec::new();
        let mut members_vec: Vec<Member> = Vec::new();

        let static_keyword: &[u8] = b"static";
        let var_keyword: &[u8] = b"var";
        let function_keyword: &[u8] = b"function";

        let mut offset = buffer_offset;

        while offset < self.buffer.len() {
            if offset + static_keyword.len() < self.buffer.len() && &self.buffer[offset..(offset + static_keyword.len())] == static_keyword {
                if self.is_next_function(offset + static_keyword.len()) {
                    let (func_offset, mut func) = self.parse_function(offset + static_keyword.len() + function_keyword.len());

                    func.is_static = true;

                    functions_vec.push(func);

                    offset = func_offset;

                    continue;
                }

                /*if self.is_next_var(offset + static_keyword.len()) {
                    offset = self.parse_var();
                    continue;
                }*/
            }

            /*if offset + var_keyword.len() < self.buffer.len() && &self.buffer[offset..var_keyword.len()] == var_keyword {

            }*/

            if offset + function_keyword.len() < self.buffer.len() && &self.buffer[offset..(offset + function_keyword.len())] == function_keyword {
                let (func_offset, func) = self.parse_function(offset + function_keyword.len());

                functions_vec.push(func);

                offset = func_offset;

                continue;
            }

            offset += 1;
        }

        (functions_vec, members_vec)
    }

    fn is_next_function(&self, offset: usize) -> bool {
        for i in offset..self.buffer.len() {
            if self.buffer[i] == b' ' {
                continue;
            }

            if self.buffer[i] != b'f' || i + 8 >= self.buffer.len() {
                break;
            }

            if &self.buffer[i..i + 8] == b"function" {
                return true;
            }
        }

        false
    }

    fn parse_function(&self, offset: usize) -> (usize, Function) {
        let mut arg_start_offset = 0;
        let mut arg_end_offset = 0;

        let mut bracket_count = 0;

        let mut function_offset = offset;

        // Parse args...
        for i in offset..self.buffer.len() {
            if self.buffer[i] == b'(' {
                arg_start_offset = i;
                continue;
            }

            if self.buffer[i] == b')' {
                arg_end_offset = i;
                break;
            }
        }

        let mut function_name: String = String::from_utf8_lossy(&self.buffer[offset..arg_start_offset]).trim().to_string();

        if function_name.contains(' ') {
            if let Some(name) = function_name.split(' ').last() {
                function_name = name.to_string();
            }
        }

        let mut args: Vec<Arg> = Vec::new();

        // Making sure we have args to parse
        if arg_end_offset - arg_start_offset > 1 {
            function_offset = arg_end_offset;

            let args_string = String::from_utf8_lossy(&self.buffer[arg_start_offset + 1..arg_end_offset]);
            let args_vec: Vec<&str> = args_string.split(',').map(|arg| arg.trim()).collect();

            for arg in args_vec {
                args.push(self.string_var_to_arg(arg.as_bytes()));
            }
        }

        // Parse impl...
        for i in function_offset..self.buffer.len() {

        }

        for i in function_offset..self.buffer.len() {
            if self.buffer[i] == b' ' {
                continue;
            }

            if self.buffer[i] == b'{' {
                bracket_count += 1;
                continue;
            }

            if self.buffer[i] == b'}' {
                bracket_count -= 1;

                if bracket_count < 1 { // function ended
                    return (i + 1, Function {
                        is_static: false,
                        name: function_name,
                        args: args,
                        lines: Vec::new(),
                    });
                }
            }
        }

        (function_offset, Function {
            is_static: false,
            name: function_name,
            args: args,
            lines: Vec::new(),
        })
    }

    fn is_next_var(&self, offset: usize) -> bool {
        for i in offset..self.buffer.len() {
            if self.buffer[i] == b' ' {
                continue;
            }

            if self.buffer[i] != b'v' || i + 3 >= self.buffer.len() {
                break;
            }

            if &self.buffer[i..i + 3] == b"var" {
                return true;
            }
        }

        false
    }

    fn parse_var(&self) -> usize {
        0
    }

    fn string_var_to_arg(&self, var: &[u8]) -> Arg {
        let mut value_start = 0;
        let mut type_start = 0;

        for i in 0..var.len() {
            if var[i] == b':' {
                type_start = i;
            }
            else if var[i] == b'=' {
                value_start = i;
                break;
            }
        }

        let has_type = type_start > 0;
        let has_value = value_start > 0;

        let name: String;
        let typ: Type;
        let value: String;

        if has_type && has_value {
            name = String::from_utf8_lossy(&var[0..type_start]).trim().to_string();
            typ = self.string_type_to_type(String::from_utf8_lossy(&var[type_start+1..value_start]).trim());
            value = String::from_utf8_lossy(&var[value_start+1..var.len()]).trim().to_string();
        }
        else if has_type && !has_value {
            name = String::from_utf8_lossy(&var[0..type_start]).trim().to_string();
            typ = self.string_type_to_type(String::from_utf8_lossy(&var[type_start+1..var.len()]).trim());
            value = String::new();
        }
        else if !has_type && has_value {
            name = String::from_utf8_lossy(&var[0..value_start]).trim().to_string();
            typ = self.detect_var_type_with_name(var);
            value = String::from_utf8_lossy(&var[value_start+1..var.len()]).trim().to_string();
        }
        else {
            name = String::from_utf8_lossy(&var[0..var.len()]).trim().to_string();
            typ = self.detect_var_type_with_name(var);
            value = String::new();
        }

        Arg {
            name: name,
            var_type: typ,
            value: value,
        }
    }

    fn is_minuscule(&self, c: u8) -> bool {
        c >= b'a' && c <= b'z'
    }

    fn is_majuscule(&self, c: u8) -> bool {
        c >= b'A' && c <= b'Z'
    }

    fn string_type_to_type(&self, typ: &str) -> Type {
        match typ {
            "String" => Type::String,
            "Number" => Type::Number,
            "Array" => Type::Array,
            "Boolean" => Type::Boolean,
            _ => Type::Object,
        }
    }

    fn char_type_to_type(&self, c: u8) -> Type {
        match c {
            b's' => Type::String,
            b'n' => Type::Number,
            b'a' => Type::Array,
            b'b' => Type::Boolean,
            _ => Type::Object,
        }
    }

    fn detect_var_type_with_name(&self, var: &[u8]) -> Type {
        if var.len() < 3 {
            if self.is_minuscule(var[0]) && self.is_majuscule(var[1]) {
                return self.char_type_to_type(var[0]);
            }
            else {
                return Type::Object;
            }
        }
        else if var[0] == b'_' && self.is_minuscule(var[1]) && self.is_majuscule(var[2]) {
            return self.char_type_to_type(var[1]);
        }
        else if self.is_minuscule(var[0]) && self.is_majuscule(var[1]) {
            return self.char_type_to_type(var[0]);
        }
        else {
            return Type::Object;
        }
    }

    fn find_unique_type(&self, keyword: &[u8]) -> (String, usize) {
        self.find_unique_type_offset(keyword, 0)
    }

    fn find_unique_type_offset(&self, keyword: &[u8], buffer_offset: usize) -> (String, usize) {
        for i in buffer_offset..self.buffer.len() {
            let max_len = i + keyword.len();

            if max_len >= self.buffer.len() {
                break;
            }

            if &self.buffer[i..max_len] == keyword {
                let mut char_start = 0;

                for offset in max_len..self.buffer.len() {
                    if self.buffer[offset] == b' ' && char_start == 0 {
                        continue;
                    }

                    if self.buffer[offset] != b' ' && char_start == 0 {
                        char_start = offset;
                        continue;
                    }

                    if self.is_not_valid_char(self.buffer[offset]) {
                        return (String::from_utf8_lossy(&self.buffer[char_start..offset]).to_owned().to_string(), offset)
                    }
                }
            }
        }

        (String::new(), buffer_offset)
    }

    //consider . has valid
    fn is_not_valid_char(&self, b: u8) -> bool {
        b == b' ' || b == b';' || b == b',' || b == b':' || b == b'=' || b == b'(' || b == b')' || b == b'\t' || b == b'\n' || b == b'\r'
    }
}