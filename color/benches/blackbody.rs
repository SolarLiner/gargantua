use criterion::*;

use color::XYZ;

fn crit_blackbody_bench(c: &mut Criterion) {
	ParameterizedBenchmark::new(
		"convert",
		|b, &s| {
			let values = lin_space(500.0, 25.0e3, s);
			b.iter(|| values.iter().map(|&t| XYZ::blackbody(t)));
		},
		vec![200, 500, 1000, 2500, 5000],
	)
	.throughput(|&s| Throughput::Elements(s as u32))
	.run("blackbody", c);
}

fn lin_space(start: f64, end: f64, length: usize) -> Vec<f64> {
	let range = end - start;
	let step = range / (length as f64);
	let mut arr: Vec<f64> = Vec::new();

	for i in 0..length {
		arr.push(start + (i as f64) * step);
	}

	return arr;
}

criterion_group!(benches, crit_blackbody_bench);
criterion_main!(benches);
