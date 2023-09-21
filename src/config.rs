use neon::prelude::*;
use neon::result::Throw;

use super::encapsulator::{self, encapsulate, Method};

pub fn module<'a>(cx: &mut ModuleContext<'a>) -> JsResult<'a, JsObject> {
    let module = cx.empty_object();

    let config_constructor = config_class::constructor(cx)?;
    module.set(cx, "Config", config_constructor)?;

    let host_constructor = host::constructor(cx)?;
    module.set(cx, "Host", host_constructor)?;

    let sample_format_obj = sample_format::object(cx)?;
    module.set(cx, "SampleFormat", sample_format_obj)?;

    Ok(module)
}

pub mod config_class {
    use super::*;

    pub fn constructor<'a, C>(cx: &mut C) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
    {
        let constructor = JsFunction::new(cx, |mut cx| {
            let output_device_js = cx.argument::<JsObject>(0)?;
            let output_config_js = cx.argument::<JsObject>(1)?;

            output_device::unpack(&mut cx, output_device_js, |cx, output_device| {
                let output_config = output_config::get(cx, output_config_js)?;
                construct(
                    cx,
                    adae::config::Config {
                        output_device: output_device.clone(),
                        output_config,
                    },
                )
            })
        })?;

        for (name, method) in STATIC_METHODS {
            let method_js = JsFunction::new(cx, *method)?;
            constructor.set(cx, *name, method_js)?;
        }

        Ok(constructor)
    }

    pub fn construct<'a, C>(cx: &mut C, config: adae::config::Config) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        encapsulate(cx, ConfigWrapper(config), &[], &[])
    }

    pub fn unpack<'a, C, F, R>(cx: &mut C, obj: Handle<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        C: Context<'a>,
        F: FnOnce(&mut C, &adae::config::Config) -> Result<R, Throw>,
    {
        encapsulator::unpack(cx, obj, |cx, config: &ConfigWrapper| {
            callback(cx, &config.0)
        })
    }

    const STATIC_METHODS: &[(&str, Method)] = &[("default", |mut cx| {
        Ok(construct(&mut cx, adae::config::Config::default())?.as_value(&mut cx))
    })];

    #[derive(Debug)]
    struct ConfigWrapper(adae::config::Config);
    impl Finalize for ConfigWrapper {}
}

mod output_config {
    use super::*;

    pub fn construct<'a, C>(
        cx: &mut C,
        output_config: adae::config::OutputConfig,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        let output_config_js = cx.empty_object();

        let channels = cx.number(output_config.channels as f64);
        let sample_format = sample_format::construct(cx, &output_config.sample_format)?;
        let sample_rate = cx.number(output_config.sample_rate as f64);

        output_config_js.set(cx, "channels", channels)?;
        output_config_js.set(cx, "sampleFormat", sample_format)?;
        output_config_js.set(cx, "sampleRate", sample_rate)?;

        if let Some(buffer_size) = output_config.buffer_size {
            let buffer_size = cx.number(buffer_size as f64);
            output_config_js.set(cx, "bufferSize", buffer_size)?;
        } else {
            let null = cx.null();
            output_config_js.set(cx, "bufferSize", null)?;
        }

        Ok(output_config_js)
    }

    pub fn get<'a, C>(
        cx: &mut C,
        obj: Handle<'a, JsObject>,
    ) -> NeonResult<adae::config::OutputConfig>
    where
        C: Context<'a>,
    {
        let output_config_js = obj.downcast_or_throw::<JsObject, _>(cx)?;

        let channels_js: Handle<JsNumber> = output_config_js.get(cx, "channels")?;
        let sample_format_js: Handle<JsString> = output_config_js.get(cx, "sampleFormat")?;
        let sample_rate_js: Handle<JsNumber> = output_config_js.get(cx, "sampleRate")?;
        let buffer_size_js: Handle<JsValue> = output_config_js.get(cx, "bufferSize")?;

        let channels_f64 = channels_js.value(cx);
        if (channels_f64 < 0.0) || ((u16::MAX as f64) < channels_f64) {
            return cx.throw_error(format!(
                "Number of channels must an integer representable as an unsigned 16-bit integer. Got {channels_f64:?}"
            ));
        }
        let channels = channels_f64 as u16;

        let sample_format = sample_format::get(cx, sample_format_js)?;

        let sample_rate_f64 = sample_rate_js.value(cx);
        if (sample_rate_f64 < 0.0) || ((u32::MAX as f64) < sample_rate_f64) {
            return cx.throw_error(format!(
                "Sample rate must be an integer representable as an unsigned 32-bit integer. Got {sample_rate_f64:?}"
            ));
        }
        let sample_rate = sample_rate_f64 as u32;

        let buffer_size = if buffer_size_js.is_a::<JsNull, _>(cx) {
            None
        } else {
            let buffer_size_number: Handle<JsNumber> = buffer_size_js.downcast_or_throw(cx)?;
            let buffer_size_f64 = buffer_size_number.value(cx);
            if (buffer_size_f64 < 0.0) || ((u32::MAX as f64) < buffer_size_f64) {
                return cx.throw_error(format!(
                    "Buffer size must be an integer representable as an unsigned 32-bit integer. Got {buffer_size_f64:?}"
                ));
            }
            Some(buffer_size_f64 as u32)
        };

        Ok(adae::config::OutputConfig {
            channels,
            sample_format,
            sample_rate,
            buffer_size,
        })
    }

    #[derive(Debug)]
    struct OutputConfigWrapper(adae::config::OutputConfig);
    impl Finalize for OutputConfigWrapper {}
}

mod output_config_range {
    use super::*;

    pub fn construct<'a, C>(
        cx: &mut C,
        output_config_range: adae::config::OutputConfigRange,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        encapsulate(
            cx,
            OutputConfigRangeWrapper(output_config_range),
            &[],
            METHODS,
        )
    }

    pub fn unpack_this<'a, F, R>(cx: &mut MethodContext<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        F: FnOnce(
            &mut MethodContext<'a, JsObject>,
            &adae::config::OutputConfigRange,
        ) -> Result<R, Throw>,
    {
        encapsulator::unpack_this(cx, |cx, range: &OutputConfigRangeWrapper| {
            callback(cx, &range.0)
        })
    }

    const METHODS: &[(&str, Method)] = &[
        ("channels", |mut cx| {
            unpack_this(&mut cx, |cx, range| {
                Ok(cx.number(range.channels() as f64).as_value(cx))
            })
        }),
        ("sampleFormat", |mut cx| {
            unpack_this(&mut cx, |cx, range| {
                let sample_format = sample_format::construct(cx, range.sample_format())?;
                Ok(sample_format.as_value(cx))
            })
        }),
        ("sampleRate", |mut cx| {
            unpack_this(&mut cx, |cx, range| {
                let range_obj = cx.empty_object();

                let min = cx.number(*range.sample_rate().start() as f64);
                let max = cx.number(*range.sample_rate().end() as f64);

                range_obj.set(cx, "min", min)?;
                range_obj.set(cx, "max", max)?;

                Ok(range_obj.as_value(cx))
            })
        }),
        ("bufferSize", |mut cx| {
            unpack_this(&mut cx, |cx, range| match range.buffer_size() {
                None => Ok(cx.null().as_value(cx)),
                Some(buffer_size) => {
                    let range_obj = cx.empty_object();

                    let min = cx.number(*buffer_size.start() as f64);
                    let max = cx.number(*buffer_size.end() as f64);

                    range_obj.set(cx, "min", min)?;
                    range_obj.set(cx, "max", max)?;

                    Ok(range_obj.as_value(cx))
                }
            })
        }),
        ("defaultConfig", |mut cx| {
            unpack_this(&mut cx, |cx, range| {
                Ok(output_config::construct(cx, range.default_config())?.as_value(cx))
            })
        }),
    ];

    #[derive(Debug)]
    struct OutputConfigRangeWrapper(adae::config::OutputConfigRange);
    impl Finalize for OutputConfigRangeWrapper {}
}

mod sample_format {
    use super::*;

    pub fn object<'a, C>(cx: &mut C) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        let obj = cx.empty_object();

        let fields = [
            ("Int8", "i8"),
            ("Int16", "i16"),
            ("Int32", "i32"),
            ("Int64", "i64"),
            ("IntUnsigned8", "u8"),
            ("IntUnsigned16", "u16"),
            ("IntUnsigned32", "u32"),
            ("IntUnsigned64", "u64"),
            ("Float32", "f32"),
            ("Float64", "f64"),
        ];

        for (name, val) in fields.iter() {
            let str = cx.string(*val);
            obj.set(cx, *name, str)?;
        }

        Ok(obj)
    }

    pub fn construct<'a, C>(
        cx: &mut C,
        sample_format: &adae::config::SampleFormat,
    ) -> JsResult<'a, JsValue>
    where
        C: Context<'a>,
    {
        use adae::config::{
            SampleFormat, SampleFormatFloat, SampleFormatInt, SampleFormatIntUnsigned,
        };
        let sample_format_str = match sample_format {
            SampleFormat::Int(x) => match x {
                SampleFormatInt::I8 => "i8",
                SampleFormatInt::I16 => "i16",
                SampleFormatInt::I32 => "i32",
                SampleFormatInt::I64 => "i64",
            },
            SampleFormat::IntUnsigned(x) => match x {
                SampleFormatIntUnsigned::U8 => "u8",
                SampleFormatIntUnsigned::U16 => "u16",
                SampleFormatIntUnsigned::U32 => "u32",
                SampleFormatIntUnsigned::U64 => "u64",
            },
            SampleFormat::Float(x) => match x {
                SampleFormatFloat::F32 => "f32",
                SampleFormatFloat::F64 => "f64",
            },
        };

        Ok(cx.string(sample_format_str).as_value(cx))
    }

    pub fn get<'a, C>(
        cx: &mut C,
        str: Handle<'a, JsString>,
    ) -> NeonResult<adae::config::SampleFormat>
    where
        C: Context<'a>,
    {
        use adae::config::{
            SampleFormat, SampleFormatFloat, SampleFormatInt, SampleFormatIntUnsigned,
        };
        let sample_format = match str.value(cx).as_str() {
            "i8" => SampleFormat::Int(SampleFormatInt::I8),
            "i16" => SampleFormat::Int(SampleFormatInt::I16),
            "i32" => SampleFormat::Int(SampleFormatInt::I32),
            "i64" => SampleFormat::Int(SampleFormatInt::I64),
            "u8" => SampleFormat::IntUnsigned(SampleFormatIntUnsigned::U8),
            "u16" => SampleFormat::IntUnsigned(SampleFormatIntUnsigned::U16),
            "u32" => SampleFormat::IntUnsigned(SampleFormatIntUnsigned::U32),
            "u64" => SampleFormat::IntUnsigned(SampleFormatIntUnsigned::U64),
            "f32" => SampleFormat::Float(SampleFormatFloat::F32),
            "f64" => SampleFormat::Float(SampleFormatFloat::F64),
            _ => return cx.throw_error(format!("Invalid sample format: {str:?}")),
        };

        Ok(sample_format)
    }
}

mod host {
    use super::*;

    pub fn constructor<'a, C>(cx: &mut C) -> JsResult<'a, JsFunction>
    where
        C: Context<'a>,
    {
        let constructor = JsFunction::new(cx, |mut cx| {
            cx.throw_error::<_, Handle<JsValue>>(
                "Host cannot be constructed directly. Use static methods instead.",
            )
        })?;

        for (name, method) in STATIC_METHODS {
            let method_js = JsFunction::new(cx, *method)?;
            constructor.set(cx, *name, method_js)?;
        }

        Ok(constructor)
    }

    pub fn construct<'a, C>(cx: &mut C, host: adae::config::Host) -> JsResult<'a, JsValue>
    where
        C: Context<'a>,
    {
        Ok(encapsulate(cx, HostWrapper(host), &[], METHODS)?.as_value(cx))
    }

    fn unpack_this<'a, F, R>(cx: &mut MethodContext<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        F: FnOnce(&mut MethodContext<'a, JsObject>, &adae::config::Host) -> Result<R, Throw>,
    {
        encapsulator::unpack_this(cx, |cx, host: &HostWrapper| callback(cx, &host.0))
    }

    const STATIC_METHODS: &[(&str, Method)] = &[
        ("available", |mut cx| {
            let available = adae::config::Host::available();
            let available_js = cx.empty_array();
            for (i, host) in available.enumerate() {
                let host_js = construct(&mut cx, host.clone())?;
                available_js.set(&mut cx, i as u32, host_js)?;
            }
            Ok(available_js.as_value(&mut cx))
        }),
        ("default", |mut cx| {
            construct(&mut cx, adae::config::Host::default())
        }),
    ];

    const METHODS: &[(&str, Method)] = &[
        ("name", |mut cx| {
            unpack_this(&mut cx, |cx, host| Ok(cx.string(host.name()).as_value(cx)))
        }),
        ("outputDevices", |mut cx| {
            unpack_this(&mut cx, |cx, host| {
                let output_devices = host
                    .output_devices()
                    .or_else(|e| cx.throw_error(format!("{e}")))?
                    .map(|device| output_device::construct(cx, device))
                    .collect::<Result<Vec<_>, _>>()?;
                let output_devices_js = cx.empty_array();
                for (i, output_device_js) in output_devices.into_iter().enumerate() {
                    output_devices_js.set(cx, i as u32, output_device_js)?;
                }
                Ok(output_devices_js.as_value(cx))
            })
        }),
        ("defaultOutputDevice", |mut cx| {
            unpack_this(&mut cx, |cx, host| {
                let output_device_opt = host
                    .default_output_device()
                    .or_else(|e| cx.throw_error(format!("{e}")))?;
                let output_device = match output_device_opt {
                    None => cx.throw_error("No default output device"),
                    Some(output_device) => Ok(output_device),
                }?;
                let output_device_js = output_device::construct(cx, output_device)?;
                Ok(output_device_js.as_value(cx))
            })
        }),
    ];

    #[derive(Debug)]
    struct HostWrapper(adae::config::Host);
    impl Finalize for HostWrapper {}
}

mod output_device {
    use super::*;

    pub fn construct<'a, C>(
        cx: &mut C,
        output_device: adae::config::OutputDevice,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        encapsulate(cx, OutputDeviceWrapper(output_device), &[], METHODS)
    }

    pub fn unpack<'a, C, F, R>(cx: &mut C, obj: Handle<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        C: Context<'a>,
        F: FnOnce(&mut C, &adae::config::OutputDevice) -> Result<R, Throw>,
    {
        encapsulator::unpack(cx, obj, |cx, device: &OutputDeviceWrapper| {
            callback(cx, &device.0)
        })
    }

    pub fn unpack_this<'a, F, R>(cx: &mut MethodContext<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        F: FnOnce(
            &mut MethodContext<'a, JsObject>,
            &adae::config::OutputDevice,
        ) -> Result<R, Throw>,
    {
        encapsulator::unpack_this(cx, |cx, device: &OutputDeviceWrapper| {
            callback(cx, &device.0)
        })
    }

    const METHODS: &[(&str, Method)] = &[
        ("host", |mut cx| {
            unpack_this(&mut cx, |cx, device| {
                Ok(host::construct(cx, device.host().clone())?.as_value(cx))
            })
        }),
        ("name", |mut cx| {
            unpack_this(&mut cx, |cx, device| {
                Ok(cx.string(device.name()).as_value(cx))
            })
        }),
        ("supportedConfigRanges", |mut cx| {
            unpack_this(&mut cx, |cx, device| {
                let supported_config_ranges = device
                    .supported_config_ranges()
                    .or_else(|e| cx.throw_error(format!("{e}")))?
                    .map(|range| output_config_range::construct(cx, range))
                    .collect::<Result<Vec<_>, _>>()?;
                let supported_config_ranges_js = cx.empty_array();
                for (i, supported_config_range_js) in
                    supported_config_ranges.into_iter().enumerate()
                {
                    supported_config_ranges_js.set(cx, i as u32, supported_config_range_js)?;
                }
                Ok(supported_config_ranges_js.as_value(cx))
            })
        }),
        ("defaultConfigRange", |mut cx| {
            unpack_this(&mut cx, |cx, device| {
                let range = device
                    .default_config_range()
                    .or_else(|e| cx.throw_error(format!("{e}")))?;

                Ok(output_config_range::construct(cx, range)?.as_value(cx))
            })
        }),
    ];

    #[derive(Debug)]
    struct OutputDeviceWrapper(adae::config::OutputDevice);
    impl Finalize for OutputDeviceWrapper {}
}
