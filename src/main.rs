use fastrand::Rng;

fn main() {
    let mut rng = Rng::new();

    let mut options = ReOptions::default();
    let mut text_alphabet: Vec<u8> = vec![];
    let mut text: Vec<u8> = vec![];
    let mut re: Vec<u8> = vec![];

    let mut test_count: u32 = 0;
    for _ in 0..1_000_000 {
        options.swarm(&mut rng, b"abcdef");
        alphabet_swarm(&mut rng, b"abcdefx", &mut text_alphabet);

        gen_re(&mut rng, &options, &mut re);

        let re = str::from_utf8(&re).unwrap();
        let r1 = regex::Regex::new(re).unwrap();
        let r2 = regex_lite::Regex::new(re).unwrap();

        for _ in 0..1000 {
            test_count += 1;
            let text =
                gen_string(&mut rng, &text_alphabet, &mut text);

            let m1 = r1.find(text)
                .map_or("not found", |it| it.as_str());
            let m2 = r2.find(text)
                .map_or("not found", |it| it.as_str());

            if m1 != m2 {
                eprintln!("err re={re} text={text} m1={m1} m2={m2}");
                return;
            }

            if test_count % 500_000 == 0 {
                eprintln!("ok  re={re} text={text}");
            }
        }
    }
}

fn alphabet_swarm<'a>(
    rng: &mut Rng,
    all: &[u8],
    pick: &'a mut Vec<u8>,
) {
    pick.clear();
    pick.extend(all);
    rng.shuffle(pick);
    let count = rng.usize(1..=pick.len());
    pick.truncate(count);
}

fn gen_string<'a>(
    rng: &mut Rng,
    alphabet: &[u8],
    result: &'a mut Vec<u8>,
) -> &'a str {
    result.clear();
    let count = rng.usize(0..8);
    for _ in 0..count {
        result.push(alphabet[rng.usize(0..alphabet.len())]);
    }
    str::from_utf8(result).unwrap()
}

#[derive(Default, Debug)]
struct ReOptions {
    alt: u16, // |
    rep: u16, // *
    any: u16, // .
    lit: u16, // 'a'
    sum: u16,
    alphabet: Vec<u8>,
}

impl ReOptions {
    fn swarm(&mut self, rng: &mut Rng, alphabet_full: &[u8]) {
        self.alt = if rng.bool() { 0 } else { rng.u16(0..100) };
        self.rep = if rng.bool() { 0 } else { rng.u16(0..100) };
        self.any = if rng.bool() { 0 } else { rng.u16(0..100) };
        self.lit = rng.u16(1..100);
        self.sum = self.alt + self.rep + self.any + self.lit;
        assert!(self.sum > 0);
        alphabet_swarm(rng, alphabet_full, &mut self.alphabet);

    }
}

fn gen_re(
    rng: &mut Rng,
    options: &ReOptions,
    result: &mut Vec<u8>,
) {
    result.clear();
    let size = rng.u8(0..8);
    gen_re_rec(rng, options, result, size);

}

fn gen_re_rec(
    rng: &mut Rng,
    options: &ReOptions,
    result: &mut Vec<u8>,
    size: u8,
) {
    if size == 0 {
        return; // Base case, empty regex.
    }

    // Pick one of the features, according to weights.
    let mut p = rng.u16(0..options.sum);
    if p < options.alt {
        // Alternation distributes the size
        // among the two children.
        let size_left = rng.u8(0..=size - 1);
        let size_right = size - size_left - 1;
        assert!(size == size_left + 1 + size_right);

        result.push(b'(');
        gen_re_rec(rng, options, result, size_left);
        result.extend(b")|(");
        gen_re_rec(rng, options, result, size_right);
        result.push(b')');
        return;
    }
    p -= options.alt;

    if p < options.rep {
        result.push(b'(');
        gen_re_rec(rng, options, result, size - 1);
        result.extend(b")*");
        return;
    }
    p -= options.rep;

    if p < options.any {
        gen_re_rec(rng, options, result, size - 1);
        result.push(b'.');
        return;
    }
    p -= options.any;

    if p < options.lit {
        gen_re_rec(rng, options, result, size - 1);
        let index = rng.usize(0..options.alphabet.len());
        let lit = options.alphabet[index];
        result.push(lit);
        return;
    }
    unreachable!();
}
