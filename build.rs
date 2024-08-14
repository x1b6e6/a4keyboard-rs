fn main() {
    libbpf_cargo::SkeletonBuilder::new()
        .source("src/bpf/write.bpf.c")
        .build_and_generate("src/bpf/write.bpf.rs")
        .unwrap();
}
