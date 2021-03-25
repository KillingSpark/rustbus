use rustbus::message_builder;
use rustbus::message_builder::MarshalledMessage;
use rustbus::params;
use rustbus::signature;

// a typedef for the complicated case
type ExecStartProp = Vec<(String, Vec<String>, bool)>;

// define the variant with a fitting marshal and unmarshal impl
rustbus::dbus_variant_sig!(TransientServiceCallProp, String => String; StringList => Vec<String>; ExecStart => ExecStartProp);

const SD1_DST: &str = "org.freedesktop.systemd1";
const SD1_PATH: &str = "/org/freedesktop/systemd1";

fn systemd_sd1_call(method: &str) -> MarshalledMessage {
    message_builder::MessageBuilder::new()
        .call(method)
        .with_interface("org.freedesktop.systemd1.Manager")
        .on(SD1_PATH)
        .at(SD1_DST)
        .build()
}

fn dbus_param_array<'a, 'e>(param: params::Array<'a, 'e>) -> params::Param<'a, 'e> {
    params::Param::Container(params::Container::Array(param))
}

fn dbus_sig(input: &str) -> signature::Type {
    signature::Type::parse_description(input).as_ref().unwrap()[0].clone()
}

fn systemd_start_transient_svc_call(
    name: String,
    args: Vec<String>,
    envs: Vec<String>,
    extra_props: Vec<(String, TransientServiceCallProp)>,
) -> MarshalledMessage {
    // NAME(s) JOB_MODE(s) PROPS(a(sv)) AUX_UNITS(a(s a(sv)))
    //
    // PROPS:
    // ["Description"] = str,
    // ["Slice"] = str,
    // ["CPUWeight"] = num,
    // ...
    // ["Environment"] = ([ENV0]=str, [ENV1]=str...)
    // ["ExecStart"] = (args[0], (args[0], args[1], ...), false)
    let mut call = systemd_sd1_call("StartTransientUnit");

    // name and job_mode
    call.body.push_param2(&name, "fail").unwrap();

    // desc string
    let desc = args.iter().fold(name.clone(), |mut a, i| {
        a += " ";
        a += i;
        a
    });

    let mut props = vec![
        (
            "Description".to_owned(),
            TransientServiceCallProp::String(desc),
        ),
        (
            "Environment".to_owned(),
            TransientServiceCallProp::StringList(envs),
        ),
        (
            "ExecStart".to_owned(),
            TransientServiceCallProp::ExecStart(vec![(args[0].clone(), args, false)]),
        ),
    ];

    for prop in extra_props.into_iter() {
        props.push(prop);
    }

    // assemble props
    call.body.push_param(props).unwrap();

    // no aux units
    call.body
        .push_old_param(&dbus_param_array(params::Array {
            element_sig: dbus_sig("(sa(sv))"),
            values: vec![],
        }))
        .unwrap();

    call
}

fn main() {
    systemd_start_transient_svc_call("ABCD".to_owned(), vec![], vec![], vec![]);
}
