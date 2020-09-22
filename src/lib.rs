use std::collections::HashMap;

const DEFAULT_SEPARATOR: &str = ": ";
const DEFAULT_NEWLINE: &str = "\n";

pub struct Serializer<'a, PairIter, ExtraIter>
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    separator: &'a str,
    newline: &'a str,
    pairs: Option<PairIter>,
    extra_lines: Option<ExtraIter>,
}

impl<'a, PairIter, ExtraIter> Serializer<'a, PairIter, ExtraIter>
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    fn new() -> Self {
        Self {
            separator: DEFAULT_SEPARATOR,
            newline: DEFAULT_NEWLINE,
            pairs: None,
            extra_lines: None,
        }
    }

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    pub fn newline(mut self, newline: &'a str) -> Self {
        self.newline = newline;
        self
    }

    pub fn pairs(mut self, pairs: PairIter) -> Self {
        self.pairs = Some(pairs);
        self
    }

    pub fn extra_lines(mut self, extra_lines: ExtraIter) -> Self {
        self.extra_lines = Some(extra_lines);
        self
    }

    pub fn serialize(self) -> String {
        let mut out = String::from("");

        if let Some(pairs) = self.pairs {
            for (k, v) in pairs {
                out.push_str(k);
                out.push_str(self.separator);
                out.push_str(v);
                out.push_str(self.newline);
            }
        }

        if let Some(extra_lines) = self.extra_lines {
            for line in extra_lines {
                out.push_str(line);
                out.push_str(self.newline);
            }
        }

        out
    }
}

pub fn serializer<'a, PairIter, ExtraIter>() -> Serializer<'a, PairIter, ExtraIter>
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    Serializer::new()
}

pub fn to_string<'a, PairIter>(iterable: PairIter) -> String
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
{
    serializer::<'a, PairIter, core::slice::Iter<&'a str>>()
        .pairs(iterable)
        .serialize()
}

pub fn serialize<'a, PairIter, ExtraIter>(pairs_iter: PairIter, extra_iter: ExtraIter) -> String
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    serializer()
        .pairs(pairs_iter)
        .extra_lines(extra_iter)
        .serialize()
}

pub struct Deserializer<'a, 'b> {
    separator: &'a str,
    newline: &'a str,
    keys: &'b [&'b str],
}

impl<'a, 'b> Deserializer<'a, 'b> {
    fn new() -> Self {
        Self {
            separator: DEFAULT_SEPARATOR,
            newline: DEFAULT_NEWLINE,
            keys: &[],
        }
    }

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    pub fn newline(mut self, newline: &'a str) -> Self {
        self.newline = newline;
        self
    }

    pub fn keys(mut self, keys: &'b [&'b str]) -> Self {
        self.keys = keys;
        self
    }

    pub fn deserialize(self, source: &'a str) -> DeserializeData<'a> {
        let mut pairs = Vec::new();
        let mut extra_lines = Vec::new();

        for line in source.lines() {
            // TODO: Use line.split_once() when stable

            let splits: Vec<_> = line.splitn(2, self.separator).collect();

            if splits.len() == 2 && self.keys.contains(&splits[0]) {
                pairs.push((splits[0], splits[1]));
            } else {
                extra_lines.push(line);
            }
        }

        DeserializeData { pairs, extra_lines }
    }
}

pub struct DeserializeData<'a> {
    pub pairs: Vec<(&'a str, &'a str)>,
    pub extra_lines: Vec<&'a str>,
}

impl<'a> DeserializeData<'a> {
    pub fn pairs_hashmap(&self) -> HashMap<&'a str, &'a str> {
        self.pairs.iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub fn pairs_hashmap_owned(&self) -> HashMap<String, String> {
        self.pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    pub fn extra_lines_vec(&self) -> Vec<&'a str> {
        self.extra_lines.clone()
    }

    pub fn extra_lines_vec_owned(&self) -> Vec<String> {
        self.extra_lines
            .iter()
            .map(|line| (*line).to_owned())
            .collect()
    }
}

pub fn deserializer<'a, 'b>() -> Deserializer<'a, 'b> {
    Deserializer::new()
}

pub fn parse<'a, 'b>(
    keys: &'b [&'b str],
    source: &'a str,
) -> (Vec<(&'a str, &'a str)>, Vec<&'a str>) {
    let DeserializeData { pairs, extra_lines } = Deserializer::new().keys(keys).deserialize(source);

    (pairs, extra_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic() {
        let ser_string = to_string([("foo", "bar"), ("baz", "123")].iter());

        let expected = "\
            foo: bar\n\
            baz: 123\n\
        ";

        assert_eq!(ser_string, expected);
    }

    #[test]
    fn test_serialize_basic2() {
        let ser_string = serialize(
            [("foo", "bar"), ("baz", "123")].iter(),
            ["extra: lines", "and stuff"].iter(),
        );

        let expected = "\
            foo: bar\n\
            baz: 123\n\
            extra: lines\n\
            and stuff\n\
        ";

        assert_eq!(ser_string, expected);
    }

    #[test]
    fn test_serialize_data() {
        let ser_string = serializer()
            .separator("=")
            .newline("\r\n")
            .pairs([("foo", "bar"), ("baz", "123")].iter())
            .extra_lines(["extra=lines", "and stuff"].iter())
            .serialize();

        let expected = "\
            foo=bar\r\n\
            baz=123\r\n\
            extra=lines\r\n\
            and stuff\r\n\
        ";

        assert_eq!(ser_string, expected);
    }

    #[test]
    fn test_deserialize_basic() {
        let source = "\
            foo: bar\n\
            baz: 123\n\
            extra: lines\n\
            and stuff\n\
        ";

        let (pairs, extra_lines) = parse(&["foo", "baz"], source);

        assert_eq!(pairs, vec![("foo", "bar"), ("baz", "123")]);
        assert_eq!(extra_lines, vec!["extra: lines", "and stuff"]);
    }

    #[test]
    fn test_deserialize_data() {
        let source = "\
            foo=bar\r\n\
            baz=123\r\n\
            extra=lines\r\n\
            and stuff\r\n\
        ";

        let data = deserializer()
            .separator("=")
            .newline("\r\n")
            .keys(&["foo", "baz"])
            .deserialize(source);

        let pairs = data.pairs;
        let extra_lines = data.extra_lines;

        assert_eq!(pairs, vec![("foo", "bar"), ("baz", "123")]);
        assert_eq!(extra_lines, vec!["extra=lines", "and stuff"]);
    }
}
