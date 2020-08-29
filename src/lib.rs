use std::io::{self, Write};
use std::iter;

const DEFAULT_SEPARATOR: &str = ": ";

pub struct SerializeData<'a, PairIter, ExtraIter>
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    pub pairs_iterable: PairIter,
    pub extra_lines_iterable: ExtraIter,
    pub separator: &'a str,
    pub newline: &'a str,
}

impl<'a, PairIter, ExtraIter> SerializeData<'a, PairIter, ExtraIter>
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    pub fn write<W: Write>(self, mut writer: W) -> io::Result<()> {
        for (k, v) in self.pairs_iterable {
            writer.write(k.as_bytes())?;
            writer.write(self.separator.as_bytes())?;
            writer.write(v.as_bytes())?;
            writer.write(self.newline.as_bytes())?;
        }

        for s in self.extra_lines_iterable {
            writer.write(self.newline.as_bytes())?;
            writer.write(s.as_bytes())?;
        }

        Ok(())
    }

    pub fn into_string(self) -> String {
        let mut out = String::from("");

        for (k, v) in self.pairs_iterable {
            out.push_str(k);
            out.push_str(self.separator);
            out.push_str(v);
            out.push_str(self.newline);
        }

        for s in self.extra_lines_iterable {
            out.push_str(s);
            out.push_str(self.newline);
        }

        out
    }
}

impl<'a, PairIter, ExtraIter> From<SerializeData<'a, PairIter, ExtraIter>> for String
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    fn from(serialize_data: SerializeData<'a, PairIter, ExtraIter>) -> Self {
        serialize_data.into_string()
    }
}

pub fn to_string_with_options<'a, PairIter, ExtraIter>(
    separator: &'a str,
    newline: &'a str,
    extra_lines: ExtraIter,
    iterable: PairIter,
) -> String
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
    ExtraIter: Iterator<Item = &'a &'a str>,
{
    SerializeData {
        pairs_iterable: iterable,
        extra_lines_iterable: extra_lines,
        separator,
        newline,
    }
    .into_string()
}

pub fn to_string<'a, PairIter>(iterable: PairIter) -> String
where
    PairIter: Iterator<Item = &'a (&'a str, &'a str)>,
{
    to_string_with_options(DEFAULT_SEPARATOR, "\n", iter::empty(), iterable)
}

pub struct DeserializeData<'a> {
    pub pairs: Vec<(&'a str, &'a str)>,
    pub extra_lines: Vec<&'a str>,
}

impl<'a> DeserializeData<'a> {
    fn new(separator: &'a str, keys: &[&'a str], source_str: &'a str) -> Self {
        let mut pairs = Vec::new();
        let mut extra_lines = Vec::new();

        for line in source_str.lines() {
            // TODO: Use line.split_once() when stable

            let splits: Vec<_> = line.splitn(2, separator).collect();

            if splits.len() == 2 && keys.contains(&splits[0]) {
                pairs.push((splits[0], splits[1]));
            } else {
                extra_lines.push(line);
            }
        }

        Self { pairs, extra_lines }
    }
}

pub fn parse_with_separator<'a>(
    separator: &'a str,
    keys: &[&'a str],
    source_str: &'a str,
) -> DeserializeData<'a> {
    DeserializeData::new(separator, keys, source_str)
}

pub fn parse<'a>(keys: &[&'a str], source_str: &'a str) -> DeserializeData<'a> {
    parse_with_separator(DEFAULT_SEPARATOR, keys, source_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_data() {
        let ser_string = to_string_with_options(
            ": ",
            "\n",
            ["extra: lines", "and stuff"].iter(),
            [("foo", "bar"), ("baz", "123")].iter(),
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
    fn test_deserialize_data() {
        let source = "\
            foo: bar\n\
            baz: 123\n\
            extra: lines\n\
            and stuff\n\
        ";

        let data = parse_with_separator(": ", &vec!["foo", "baz"], source);

        assert_eq!(data.pairs, vec![("foo", "bar"), ("baz", "123")]);
        assert_eq!(data.extra_lines, vec!["extra: lines", "and stuff"]);
    }
}
