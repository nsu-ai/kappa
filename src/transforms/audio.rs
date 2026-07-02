// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyModule};

fn torchaudio_transforms(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    py.import("torchaudio.transforms")
}

fn instantiate_transform_with_kwargs(
    py: Python<'_>,
    class_name: &str,
    kwargs: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let transforms_mod = torchaudio_transforms(py)?;
    let cls = transforms_mod.getattr(class_name)?;
    let obj = cls.call((), Some(kwargs))?;
    Ok(obj.unbind())
}

#[pyclass(unsendable)]
pub struct Spectrogram {
    inner: Py<PyAny>,
}

#[pymethods]
impl Spectrogram {
    #[new]
    #[pyo3(signature = (
        n_fft=400,
        win_length=None,
        hop_length=None,
        pad=0,
        window_fn=None,
        power=2.0,
        normalized=false,
        wkwargs=None
    ))]
    pub fn new(
        py: Python<'_>,
        n_fft: i32,
        win_length: Option<i32>,
        hop_length: Option<i32>,
        pad: i32,
        window_fn: Option<Py<PyAny>>,
        power: f32,
        normalized: bool,
        wkwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_fft", n_fft)?;
        if let Some(v) = win_length {
            kwargs.set_item("win_length", v)?;
        }
        if let Some(v) = hop_length {
            kwargs.set_item("hop_length", v)?;
        }
        kwargs.set_item("pad", pad)?;
        if let Some(v) = window_fn {
            kwargs.set_item("window_fn", v)?;
        }
        kwargs.set_item("power", power)?;
        kwargs.set_item("normalized", normalized)?;
        if let Some(v) = wkwargs {
            kwargs.set_item("wkwargs", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "Spectrogram", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct GriffinLim {
    inner: Py<PyAny>,
}

#[pymethods]
impl GriffinLim {
    #[new]
    #[pyo3(signature = (
        n_fft=400,
        n_iter=32,
        win_length=None,
        hop_length=None,
        window_fn=None,
        power=2.0,
        normalized=false,
        wkwargs=None,
        momentum=0.99,
        length=None,
        rand_init=true
    ))]
    pub fn new(
        py: Python<'_>,
        n_fft: i32,
        n_iter: i32,
        win_length: Option<i32>,
        hop_length: Option<i32>,
        window_fn: Option<Py<PyAny>>,
        power: f32,
        normalized: bool,
        wkwargs: Option<Py<PyAny>>,
        momentum: f32,
        length: Option<i32>,
        rand_init: bool,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_fft", n_fft)?;
        kwargs.set_item("n_iter", n_iter)?;
        if let Some(v) = win_length {
            kwargs.set_item("win_length", v)?;
        }
        if let Some(v) = hop_length {
            kwargs.set_item("hop_length", v)?;
        }
        if let Some(v) = window_fn {
            kwargs.set_item("window_fn", v)?;
        }
        kwargs.set_item("power", power)?;
        kwargs.set_item("normalized", normalized)?;
        if let Some(v) = wkwargs {
            kwargs.set_item("wkwargs", v)?;
        }
        kwargs.set_item("momentum", momentum)?;
        if let Some(v) = length {
            kwargs.set_item("length", v)?;
        }
        kwargs.set_item("rand_init", rand_init)?;
        let inner = instantiate_transform_with_kwargs(py, "GriffinLim", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, specgram: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((specgram,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct AmplitudeToDB {
    inner: Py<PyAny>,
}

#[pymethods]
impl AmplitudeToDB {
    #[new]
    #[pyo3(signature = (stype=None, top_db=None))]
    pub fn new(py: Python<'_>, stype: Option<String>, top_db: Option<f32>) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("stype", stype.unwrap_or_else(|| "power".to_string()))?;
        if let Some(v) = top_db {
            kwargs.set_item("top_db", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "AmplitudeToDB", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, x: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((x,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MelScale {
    inner: Py<PyAny>,
}

#[pymethods]
impl MelScale {
    #[new]
    #[pyo3(signature = (
        n_mels=128,
        sample_rate=16000,
        f_min=0.0,
        f_max=None,
        n_stft=None
    ))]
    pub fn new(
        py: Python<'_>,
        n_mels: i32,
        sample_rate: i32,
        f_min: f32,
        f_max: Option<f32>,
        n_stft: Option<i32>,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_mels", n_mels)?;
        kwargs.set_item("sample_rate", sample_rate)?;
        kwargs.set_item("f_min", f_min)?;
        if let Some(v) = f_max {
            kwargs.set_item("f_max", v)?;
        }
        if let Some(v) = n_stft {
            kwargs.set_item("n_stft", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "MelScale", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, specgram: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((specgram,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct InverseMelScale {
    inner: Py<PyAny>,
}

#[pymethods]
impl InverseMelScale {
    #[new]
    #[pyo3(signature = (
        n_stft,
        n_mels=128,
        sample_rate=16000,
        f_min=0.0,
        f_max=None,
        max_iter=100000,
        tolerance_loss=1e-5,
        tolerance_change=1e-8,
        sgdargs=None
    ))]
    pub fn new(
        py: Python<'_>,
        n_stft: i32,
        n_mels: i32,
        sample_rate: i32,
        f_min: f32,
        f_max: Option<f32>,
        max_iter: i32,
        tolerance_loss: f32,
        tolerance_change: f32,
        sgdargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_stft", n_stft)?;
        kwargs.set_item("n_mels", n_mels)?;
        kwargs.set_item("sample_rate", sample_rate)?;
        kwargs.set_item("f_min", f_min)?;
        if let Some(v) = f_max {
            kwargs.set_item("f_max", v)?;
        }
        kwargs.set_item("max_iter", max_iter)?;
        kwargs.set_item("tolerance_loss", tolerance_loss)?;
        kwargs.set_item("tolerance_change", tolerance_change)?;
        if let Some(v) = sgdargs {
            kwargs.set_item("sgdargs", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "InverseMelScale", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, melspec: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((melspec,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MelSpectrogram {
    inner: Py<PyAny>,
}

#[pymethods]
impl MelSpectrogram {
    #[new]
    #[pyo3(signature = (
        sample_rate=16000,
        n_fft=400,
        win_length=None,
        hop_length=None,
        f_min=0.0,
        f_max=None,
        pad=0,
        n_mels=128,
        window_fn=None,
        power=2.0,
        normalized=false,
        wkwargs=None
    ))]
    pub fn new(
        py: Python<'_>,
        sample_rate: i32,
        n_fft: i32,
        win_length: Option<i32>,
        hop_length: Option<i32>,
        f_min: f32,
        f_max: Option<f32>,
        pad: i32,
        n_mels: i32,
        window_fn: Option<Py<PyAny>>,
        power: f32,
        normalized: bool,
        wkwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("sample_rate", sample_rate)?;
        kwargs.set_item("n_fft", n_fft)?;
        if let Some(v) = win_length {
            kwargs.set_item("win_length", v)?;
        }
        if let Some(v) = hop_length {
            kwargs.set_item("hop_length", v)?;
        }
        kwargs.set_item("f_min", f_min)?;
        if let Some(v) = f_max {
            kwargs.set_item("f_max", v)?;
        }
        kwargs.set_item("pad", pad)?;
        kwargs.set_item("n_mels", n_mels)?;
        if let Some(v) = window_fn {
            kwargs.set_item("window_fn", v)?;
        }
        kwargs.set_item("power", power)?;
        kwargs.set_item("normalized", normalized)?;
        if let Some(v) = wkwargs {
            kwargs.set_item("wkwargs", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "MelSpectrogram", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MFCC {
    inner: Py<PyAny>,
}

#[pymethods]
impl MFCC {
    #[new]
    #[pyo3(signature = (
        sample_rate=16000,
        n_mfcc=40,
        dct_type=2,
        norm=None,
        log_mels=false,
        melkwargs=None
    ))]
    pub fn new(
        py: Python<'_>,
        sample_rate: i32,
        n_mfcc: i32,
        dct_type: i32,
        norm: Option<String>,
        log_mels: bool,
        melkwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let norm = norm.unwrap_or_else(|| "ortho".to_string());
        let kwargs = PyDict::new(py);
        kwargs.set_item("sample_rate", sample_rate)?;
        kwargs.set_item("n_mfcc", n_mfcc)?;
        kwargs.set_item("dct_type", dct_type)?;
        kwargs.set_item("norm", norm)?;
        kwargs.set_item("log_mels", log_mels)?;
        if let Some(v) = melkwargs {
            kwargs.set_item("melkwargs", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "MFCC", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MuLawEncoding {
    inner: Py<PyAny>,
}

#[pymethods]
impl MuLawEncoding {
    #[new]
    #[pyo3(signature = (quantization_channels=256))]
    pub fn new(py: Python<'_>, quantization_channels: i32) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("quantization_channels", quantization_channels)?;
        let inner = instantiate_transform_with_kwargs(py, "MuLawEncoding", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, x: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((x,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MuLawDecoding {
    inner: Py<PyAny>,
}

#[pymethods]
impl MuLawDecoding {
    #[new]
    #[pyo3(signature = (quantization_channels=256))]
    pub fn new(py: Python<'_>, quantization_channels: i32) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("quantization_channels", quantization_channels)?;
        let inner = instantiate_transform_with_kwargs(py, "MuLawDecoding", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, x_mu: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((x_mu,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct Resample {
    inner: Py<PyAny>,
}

#[pymethods]
impl Resample {
    #[new]
    #[pyo3(signature = (
        orig_freq=16000,
        new_freq=16000,
        resampling_method=None
    ))]
    pub fn new(
        py: Python<'_>,
        orig_freq: i32,
        new_freq: i32,
        resampling_method: Option<String>,
    ) -> PyResult<Self> {
        let resampling_method =
            resampling_method.unwrap_or_else(|| "sinc_interpolation".to_string());
        let kwargs = PyDict::new(py);
        kwargs.set_item("orig_freq", orig_freq)?;
        kwargs.set_item("new_freq", new_freq)?;
        kwargs.set_item("resampling_method", resampling_method)?;
        let inner = instantiate_transform_with_kwargs(py, "Resample", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct ComplexNorm {
    inner: Py<PyAny>,
}

#[pymethods]
impl ComplexNorm {
    #[new]
    #[pyo3(signature = (power=1.0))]
    pub fn new(py: Python<'_>, power: f32) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("power", power)?;
        let inner = instantiate_transform_with_kwargs(py, "ComplexNorm", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, complex_tensor: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((complex_tensor,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct TimeStretch {
    inner: Py<PyAny>,
}

#[pymethods]
impl TimeStretch {
    #[new]
    #[pyo3(signature = (hop_length=None, n_freq=201, fixed_rate=None))]
    pub fn new(
        py: Python<'_>,
        hop_length: Option<i32>,
        n_freq: i32,
        fixed_rate: Option<f32>,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        if let Some(v) = hop_length {
            kwargs.set_item("hop_length", v)?;
        }
        kwargs.set_item("n_freq", n_freq)?;
        if let Some(v) = fixed_rate {
            kwargs.set_item("fixed_rate", v)?;
        }
        let inner = instantiate_transform_with_kwargs(py, "TimeStretch", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(
        &self,
        py: Python<'_>,
        complex_specgrams: PyObject,
        overriding_rate: Option<f32>,
    ) -> PyResult<PyObject> {
        let bound = self.inner.bind(py);
        let out = if let Some(r) = overriding_rate {
            bound.call1((complex_specgrams, r))?
        } else {
            bound.call1((complex_specgrams,))?
        };
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct Fade {
    inner: Py<PyAny>,
}

#[pymethods]
impl Fade {
    #[new]
    #[pyo3(signature = (fade_in_len=0, fade_out_len=0, fade_shape=None))]
    pub fn new(
        py: Python<'_>,
        fade_in_len: i32,
        fade_out_len: i32,
        fade_shape: Option<String>,
    ) -> PyResult<Self> {
        let fade_shape = fade_shape.unwrap_or_else(|| "linear".to_string());
        let kwargs = PyDict::new(py);
        kwargs.set_item("fade_in_len", fade_in_len)?;
        kwargs.set_item("fade_out_len", fade_out_len)?;
        kwargs.set_item("fade_shape", fade_shape)?;
        let inner = instantiate_transform_with_kwargs(py, "Fade", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct FrequencyMasking {
    inner: Py<PyAny>,
}

#[pymethods]
impl FrequencyMasking {
    #[new]
    #[pyo3(signature = (freq_mask_param, iid_masks=false))]
    pub fn new(py: Python<'_>, freq_mask_param: i32, iid_masks: bool) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("freq_mask_param", freq_mask_param)?;
        kwargs.set_item("iid_masks", iid_masks)?;
        let inner = instantiate_transform_with_kwargs(py, "FrequencyMasking", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, specgram: PyObject, mask_value: Option<f32>) -> PyResult<PyObject> {
        let bound = self.inner.bind(py);
        let out = if let Some(v) = mask_value {
            bound.call1((specgram, v))?
        } else {
            bound.call1((specgram,))?
        };
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct TimeMasking {
    inner: Py<PyAny>,
}

#[pymethods]
impl TimeMasking {
    #[new]
    #[pyo3(signature = (time_mask_param, iid_masks=false))]
    pub fn new(py: Python<'_>, time_mask_param: i32, iid_masks: bool) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("time_mask_param", time_mask_param)?;
        kwargs.set_item("iid_masks", iid_masks)?;
        let inner = instantiate_transform_with_kwargs(py, "TimeMasking", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, specgram: PyObject, mask_value: Option<f32>) -> PyResult<PyObject> {
        let bound = self.inner.bind(py);
        let out = if let Some(v) = mask_value {
            bound.call1((specgram, v))?
        } else {
            bound.call1((specgram,))?
        };
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct SlidingWindowCmn {
    inner: Py<PyAny>,
}

#[pymethods]
impl SlidingWindowCmn {
    #[new]
    #[pyo3(signature = (cmn_window=600, min_cmn_window=100, center=false, norm_vars=false))]
    pub fn new(
        py: Python<'_>,
        cmn_window: i32,
        min_cmn_window: i32,
        center: bool,
        norm_vars: bool,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("cmn_window", cmn_window)?;
        kwargs.set_item("min_cmn_window", min_cmn_window)?;
        kwargs.set_item("center", center)?;
        kwargs.set_item("norm_vars", norm_vars)?;
        let inner = instantiate_transform_with_kwargs(py, "SlidingWindowCmn", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct Vad {
    inner: Py<PyAny>,
}

#[pymethods]
impl Vad {
    #[new]
    #[pyo3(signature = (
        sample_rate,
        trigger_level=7.0,
        trigger_time=0.25,
        search_time=1.0,
        allowed_gap=0.25,
        pre_trigger_time=0.0,
        boot_time=0.35,
        noise_up_time=0.1,
        noise_down_time=0.01,
        noise_reduction_amount=1.35,
        measure_freq=20.0,
        measure_duration=None,
        measure_smooth_time=0.4,
        hp_filter_freq=50.0,
        lp_filter_freq=6000.0,
        hp_lifter_freq=150.0,
        lp_lifter_freq=2000.0
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        py: Python<'_>,
        sample_rate: i32,
        trigger_level: f32,
        trigger_time: f32,
        search_time: f32,
        allowed_gap: f32,
        pre_trigger_time: f32,
        boot_time: f32,
        noise_up_time: f32,
        noise_down_time: f32,
        noise_reduction_amount: f32,
        measure_freq: f32,
        measure_duration: Option<f32>,
        measure_smooth_time: f32,
        hp_filter_freq: f32,
        lp_filter_freq: f32,
        hp_lifter_freq: f32,
        lp_lifter_freq: f32,
    ) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("sample_rate", sample_rate)?;
        kwargs.set_item("trigger_level", trigger_level)?;
        kwargs.set_item("trigger_time", trigger_time)?;
        kwargs.set_item("search_time", search_time)?;
        kwargs.set_item("allowed_gap", allowed_gap)?;
        kwargs.set_item("pre_trigger_time", pre_trigger_time)?;
        kwargs.set_item("boot_time", boot_time)?;
        kwargs.set_item("noise_up_time", noise_up_time)?;
        kwargs.set_item("noise_down_time", noise_down_time)?;
        kwargs.set_item("noise_reduction_amount", noise_reduction_amount)?;
        kwargs.set_item("measure_freq", measure_freq)?;
        if let Some(v) = measure_duration {
            kwargs.set_item("measure_duration", v)?;
        }
        kwargs.set_item("measure_smooth_time", measure_smooth_time)?;
        kwargs.set_item("hp_filter_freq", hp_filter_freq)?;
        kwargs.set_item("lp_filter_freq", lp_filter_freq)?;
        kwargs.set_item("hp_lifter_freq", hp_lifter_freq)?;
        kwargs.set_item("lp_lifter_freq", lp_lifter_freq)?;
        let inner = instantiate_transform_with_kwargs(py, "Vad", &kwargs)?;
        Ok(Self { inner })
    }

    fn __call__(&self, py: Python<'_>, waveform: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((waveform,))?;
        Ok(out.unbind().into())
    }
}

/// Register torchaudio transforms implemented via Python delegation.
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Spectrogram>()?;
    module.add_class::<GriffinLim>()?;
    module.add_class::<AmplitudeToDB>()?;
    module.add_class::<MelScale>()?;
    module.add_class::<InverseMelScale>()?;
    module.add_class::<MelSpectrogram>()?;
    module.add_class::<MFCC>()?;
    module.add_class::<MuLawEncoding>()?;
    module.add_class::<MuLawDecoding>()?;
    module.add_class::<Resample>()?;
    module.add_class::<ComplexNorm>()?;
    module.add_class::<TimeStretch>()?;
    module.add_class::<Fade>()?;
    module.add_class::<FrequencyMasking>()?;
    module.add_class::<TimeMasking>()?;
    module.add_class::<SlidingWindowCmn>()?;
    module.add_class::<Vad>()?;
    Ok(())
}