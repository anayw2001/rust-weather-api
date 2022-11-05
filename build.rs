use protobuf_codegen::Customize;

fn main() {
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("proto")
        .input("proto/weather_message.proto")
        .cargo_out_dir("proto")
        .customize(Customize::default().gen_mod_rs(true).generate_getter(true))
        .run_from_script();
}
