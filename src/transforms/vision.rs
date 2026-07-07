// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PySequence, PyTuple};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

fn is_python_dict(value: &Bound<'_, PyAny>) -> bool {
    value.downcast::<PyDict>().is_ok()
}

fn extract_first_file_path_from_sample(sample: &Bound<'_, PyDict>) -> PyResult<String> {
    let files_value = sample
        .get_item("files")?
        .ok_or_else(|| PyErr::new::<PyValueError, _>("Sample is missing key 'files'"))?;

    if files_value.is_none() {
        return Err(PyErr::new::<PyValueError, _>(
            "Sample['files'] is None; expected a list of ItemFile objects",
        ));
    }

    let files_sequence: &Bound<'_, PySequence> = files_value.downcast::<PySequence>().map_err(|_| {
        PyErr::new::<PyValueError, _>("Sample['files'] must be a sequence of ItemFile objects")
    })?;

    let first = files_sequence.get_item(0).map_err(|_| {
        PyErr::new::<PyValueError, _>(
            "Sample['files'] is empty; expected at least one file",
        )
    })?;

    let file_path: String = first
        .getattr("file")
        .and_then(|v| v.extract::<String>())
        .map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "Sample['files'][0] must have a string attribute 'file' (path)",
            )
        })?;

    Ok(file_path)
}

fn ensure_pil_image<'py>(py: Python<'py>, value: &Bound<'py, PyAny>) -> PyResult<PyObject> {
    if value.hasattr("resize")? && value.hasattr("crop")? {
        return Ok(value.clone().unbind().into());
    }

    if value.is_instance_of::<PyDict>() {
        let sample = value.downcast::<PyDict>()?;
        if let Some(existing) = sample.get_item("image")?
            && existing.hasattr("resize")? && existing.hasattr("crop")?
        {
            return Ok(existing.unbind().into());
        }

        let file_path = extract_first_file_path_from_sample(sample)?;
        let pil = py.import("PIL.Image").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "PIL is required for vision transforms (pip install pillow)",
            )
        })?;
        let image = pil.call_method1("open", (file_path,))?;
        let rgb_image = image.call_method1("convert", ("RGB",))?;
        sample.set_item("image", rgb_image.clone())?;
        return Ok(rgb_image.unbind().into());
    }

    if let Ok(path) = value.extract::<String>() {
        let pil = py.import("PIL.Image").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "PIL is required for vision transforms (pip install pillow)",
            )
        })?;
        let image = pil.call_method1("open", (path,))?;
        let rgb_image = image.call_method1("convert", ("RGB",))?;
        return Ok(rgb_image.unbind().into());
    }

    Err(PyErr::new::<PyValueError, _>(
        "Expected a PIL.Image, a sample dict, or an image path string",
    ))
}

fn set_sample_image_if_dict(py: Python<'_>, maybe_sample: &Bound<'_, PyAny>, image: &PyObject) -> PyResult<()> {
    if is_python_dict(maybe_sample) {
        let sample = maybe_sample.downcast::<PyDict>()?;
        sample.set_item("image", image.clone_ref(py))?;
    }
    Ok(())
}

/// A torchvision-like `Compose` transform.
///
/// It can be used as a dataset `transform` callable:
/// - If called with a sample dict, it will pass that dict through all transforms.
/// - If called with an image/tensor, it will pass that object through all transforms.
#[pyclass]
pub struct Compose {
    transforms: Vec<PyObject>,
}

#[pymethods]
impl Compose {
    #[new]
    pub fn new(transforms: Vec<PyObject>) -> Self {
        Self { transforms }
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let mut value = input;
        for transform in &self.transforms {
            let transformed = transform.call1(py, (value,))?;
            value = transformed;
        }
        Ok(value)
    }
}

/// Load an image for a sample dict and store it under `sample["image"]`.
///
/// - If input is a dict: reads the first file in `sample["files"]`.
/// - If input is a string path: loads it.
/// - If input is already a PIL image: returns it unchanged.
#[pyclass]
pub struct LoadImage;

#[pymethods]
impl LoadImage {
    #[new]
    pub fn new() -> Self {
        Self
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        set_sample_image_if_dict(py, &bound, &image)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(image)
        }
    }
}

/// Resize a PIL image to a given `(height, width)`.
///
/// If called with a sample dict, it reads/writes `sample["image"]`.
#[pyclass]
pub struct Resize {
    height: u32,
    width: u32,
}

#[pymethods]
impl Resize {
    #[new]
    pub fn new(height: u32, width: u32) -> Self {
        Self { height, width }
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let size = PyTuple::new(py, [self.width, self.height])?;
        let resized = image_bound.call_method1("resize", (size,))?;
        let resized_obj: PyObject = resized.into();

        set_sample_image_if_dict(py, &bound, &resized_obj)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(resized_obj)
        }
    }
}

/// Center crop a PIL image to `(height, width)`.
#[pyclass]
pub struct CenterCrop {
    height: u32,
    width: u32,
}

#[pymethods]
impl CenterCrop {
    #[new]
    pub fn new(height: u32, width: u32) -> Self {
        Self { height, width }
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let (image_width, image_height): (u32, u32) = image_bound
            .getattr("size")?
            .extract()
            .map_err(|_| PyErr::new::<PyValueError, _>("PIL image has invalid 'size'"))?;

        if self.width > image_width || self.height > image_height {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "CenterCrop size ({}, {}) exceeds image size ({}, {})",
                self.height, self.width, image_height, image_width
            )));
        }

        let left = ((image_width - self.width) / 2) as i64;
        let upper = ((image_height - self.height) / 2) as i64;
        let right = (left as u32 + self.width) as i64;
        let lower = (upper as u32 + self.height) as i64;

        let crop_box = PyTuple::new(py, [left, upper, right, lower])?;
        let cropped = image_bound.call_method1("crop", (crop_box,))?;
        let cropped_obj: PyObject = cropped.into();

        set_sample_image_if_dict(py, &bound, &cropped_obj)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(cropped_obj)
        }
    }
}

/// Convert a PIL image to a float32 torch tensor in CHW layout, scaled to \([0, 1]\).
///
/// - If input is a sample dict, it reads `sample["image"]` and writes `sample["tensor"]`.
/// - If input is a PIL image, returns a tensor.
#[pyclass]
pub struct ToTensor;

#[pymethods]
impl ToTensor {
    #[new]
    pub fn new() -> Self {
        Self
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let numpy = py.import("numpy").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "numpy is required for ToTensor (pip install numpy)",
            )
        })?;
        let torch = py.import("torch").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "torch is required for ToTensor (pip install torch)",
            )
        })?;

        let array = numpy.call_method1("array", (image_bound,))?;
        let tensor = torch.call_method1("from_numpy", (array,))?;
        let tensor = tensor.call_method1("permute", (2_u8, 0_u8, 1_u8))?;
        let tensor = tensor.call_method1("to", ("float32",))?;
        let tensor = tensor.call_method1("__truediv__", (255.0_f32,))?;
        let tensor_obj: PyObject = tensor.into();

        if is_python_dict(&bound) {
            let sample = bound.downcast::<PyDict>()?;
            sample.set_item("tensor", tensor_obj.clone_ref(py))?;
            Ok(input)
        } else {
            Ok(tensor_obj)
        }
    }
}

/// Normalize a torch tensor with per-channel mean/std (CHW).
///
/// - If input is a sample dict, it reads `sample["tensor"]` and writes it back.
/// - If input is a tensor, returns the normalized tensor.
#[pyclass]
pub struct Normalize {
    mean: Vec<f32>,
    std: Vec<f32>,
}

#[pymethods]
impl Normalize {
    #[new]
    pub fn new(mean: Vec<f32>, std: Vec<f32>) -> PyResult<Self> {
        if mean.len() != std.len() {
            return Err(PyErr::new::<PyValueError, _>(
                "Normalize requires mean and std with same length",
            ));
        }
        if mean.is_empty() {
            return Err(PyErr::new::<PyValueError, _>(
                "Normalize requires non-empty mean/std",
            ));
        }
        Ok(Self { mean, std })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);

        let tensor: Bound<'_, PyAny> = if is_python_dict(&bound) {
            let sample = bound.downcast::<PyDict>()?;
            let tensor_value = sample.get_item("tensor")?.ok_or_else(|| {
                PyErr::new::<PyValueError, _>(
                    "Sample is missing key 'tensor'; run ToTensor first",
                )
            })?;
            tensor_value
        } else {
            bound.clone()
        };

        let torch = py.import("torch").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "torch is required for Normalize (pip install torch)",
            )
        })?;

        let mean_tensor = torch.call_method1("tensor", (self.mean.clone(),))?;
        let std_tensor = torch.call_method1("tensor", (self.std.clone(),))?;

        let mean_tensor = mean_tensor.call_method1("view", (-1_i32, 1_i32, 1_i32))?;
        let std_tensor = std_tensor.call_method1("view", (-1_i32, 1_i32, 1_i32))?;

        let normalized = tensor.call_method1("__sub__", (mean_tensor,))?;
        let normalized = normalized.call_method1("__truediv__", (std_tensor,))?;
        let normalized_obj: PyObject = normalized.into();

        if is_python_dict(&bound) {
            let sample = bound.downcast::<PyDict>()?;
            sample.set_item("tensor", normalized_obj.clone_ref(py))?;
            Ok(input)
        } else {
            Ok(normalized_obj)
        }
    }
}

/// Randomly flip a PIL image horizontally with probability `p`.
#[pyclass(unsendable)]
pub struct RandomHorizontalFlip {
    p: f32,
    rng: RefCell<Option<StdRng>>,
}

#[pymethods]
impl RandomHorizontalFlip {
    #[new]
    #[pyo3(signature = (p=0.5, seed=None))]
    pub fn new(p: f32, seed: Option<u64>) -> PyResult<Self> {
        if !(0.0..=1.0).contains(&p) {
            return Err(PyErr::new::<PyValueError, _>(
                "RandomHorizontalFlip requires p in [0, 1]",
            ));
        }
        let rng = seed.map(StdRng::seed_from_u64);
        Ok(Self {
            p,
            rng: RefCell::new(rng),
        })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let mut rng_ref = self.rng.borrow_mut();
        let random_value: f32 = if let Some(rng) = rng_ref.as_mut() {
            rng.r#gen()
        } else {
            rand::thread_rng().r#gen()
        };

        if random_value >= self.p {
            return Ok(input);
        }

        let pil_image_module = py.import("PIL.Image").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "PIL is required for vision transforms (pip install pillow)",
            )
        })?;
        let transpose_enum = pil_image_module.getattr("Transpose")?;
        let flip_left_right = transpose_enum.getattr("FLIP_LEFT_RIGHT")?;

        let flipped = image_bound.call_method1("transpose", (flip_left_right,))?;
        let flipped_obj: PyObject = flipped.into();

        set_sample_image_if_dict(py, &bound, &flipped_obj)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(flipped_obj)
        }
    }
}

/// Randomly flip a PIL image vertically with probability `p`.
#[pyclass(unsendable)]
pub struct RandomVerticalFlip {
    p: f32,
    rng: RefCell<Option<StdRng>>,
}

#[pymethods]
impl RandomVerticalFlip {
    #[new]
    #[pyo3(signature = (p=0.5, seed=None))]
    pub fn new(p: f32, seed: Option<u64>) -> PyResult<Self> {
        if !(0.0..=1.0).contains(&p) {
            return Err(PyErr::new::<PyValueError, _>(
                "RandomVerticalFlip requires p in [0, 1]",
            ));
        }
        let rng = seed.map(StdRng::seed_from_u64);
        Ok(Self {
            p,
            rng: RefCell::new(rng),
        })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let mut rng_ref = self.rng.borrow_mut();
        let random_value: f32 = if let Some(rng) = rng_ref.as_mut() {
            rng.r#gen()
        } else {
            rand::thread_rng().r#gen()
        };

        if random_value >= self.p {
            return Ok(input);
        }

        let pil_image_module = py.import("PIL.Image").map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "PIL is required for vision transforms (pip install pillow)",
            )
        })?;
        let transpose_enum = pil_image_module.getattr("Transpose")?;
        let flip_top_bottom = transpose_enum.getattr("FLIP_TOP_BOTTOM")?;

        let flipped = image_bound.call_method1("transpose", (flip_top_bottom,))?;
        let flipped_obj: PyObject = flipped.into();

        set_sample_image_if_dict(py, &bound, &flipped_obj)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(flipped_obj)
        }
    }
}

/// Randomly crop a PIL image to `(height, width)`.
#[pyclass(unsendable)]
pub struct RandomCrop {
    height: u32,
    width: u32,
    rng: RefCell<Option<StdRng>>,
}

#[pymethods]
impl RandomCrop {
    #[new]
    #[pyo3(signature = (height, width, seed=None))]
    pub fn new(height: u32, width: u32, seed: Option<u64>) -> PyResult<Self> {
        if height == 0 || width == 0 {
            return Err(PyErr::new::<PyValueError, _>(
                "RandomCrop requires non-zero height and width",
            ));
        }
        let rng = seed.map(StdRng::seed_from_u64);
        Ok(Self {
            height,
            width,
            rng: RefCell::new(rng),
        })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let bound = input.bind(py);
        let image = ensure_pil_image(py, &bound)?;
        let image_bound = image.bind(py);

        let (image_width, image_height): (u32, u32) = image_bound
            .getattr("size")?
            .extract()
            .map_err(|_| PyErr::new::<PyValueError, _>("PIL image has invalid 'size'"))?;

        if self.width > image_width || self.height > image_height {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "RandomCrop size ({}, {}) exceeds image size ({}, {})",
                self.height, self.width, image_height, image_width
            )));
        }

        let max_left = image_width - self.width;
        let max_upper = image_height - self.height;

        let mut rng_ref = self.rng.borrow_mut();
        let (left, upper): (u32, u32) = if let Some(rng) = rng_ref.as_mut() {
            (rng.gen_range(0..=max_left), rng.gen_range(0..=max_upper))
        } else {
            let mut rng = rand::thread_rng();
            (rng.gen_range(0..=max_left), rng.gen_range(0..=max_upper))
        };

        let right = left + self.width;
        let lower = upper + self.height;

        let crop_box = PyTuple::new(py, [left as i64, upper as i64, right as i64, lower as i64])?;
        let cropped = image_bound.call_method1("crop", (crop_box,))?;
        let cropped_obj: PyObject = cropped.into();

        set_sample_image_if_dict(py, &bound, &cropped_obj)?;
        if is_python_dict(&bound) {
            Ok(input)
        } else {
            Ok(cropped_obj)
        }
    }
}

/// Register vision transform classes into a Python module.
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Compose>()?;
    module.add_class::<LoadImage>()?;
    module.add_class::<Resize>()?;
    module.add_class::<CenterCrop>()?;
    module.add_class::<ToTensor>()?;
    module.add_class::<Normalize>()?;
    module.add_class::<RandomHorizontalFlip>()?;
    module.add_class::<RandomVerticalFlip>()?;
    module.add_class::<RandomCrop>()?;
    Ok(())
}