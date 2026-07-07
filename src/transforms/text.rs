// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyModule, PyTuple};

fn torchtext_transforms(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    py.import("torchtext.transforms")
}

#[pyclass(unsendable)]
pub struct SentencePieceTokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl SentencePieceTokenizer {
    #[new]
    pub fn new(py: Python<'_>, sp_model_path: String) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("SentencePieceTokenizer")?;
        let inner = cls.call1((sp_model_path,))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct VocabTransform {
    inner: Py<PyAny>,
}

#[pymethods]
impl VocabTransform {
    #[new]
    pub fn new(py: Python<'_>, vocab: Py<PyAny>) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("VocabTransform")?;
        let inner = cls.call1((vocab.clone_ref(py),))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct ToTensor {
    inner: Py<PyAny>,
}

#[pymethods]
impl ToTensor {
    #[new]
    #[pyo3(signature = (padding_value=None, dtype=None))]
    pub fn new(
        py: Python<'_>,
        padding_value: Option<i32>,
        dtype: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("ToTensor")?;
        let inner = if let Some(dtype) = dtype {
            cls.call1((padding_value, dtype.clone_ref(py)))?
        } else {
            cls.call1((padding_value,))?
        };
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct LabelToIndex {
    inner: Py<PyAny>,
}

#[pymethods]
impl LabelToIndex {
    #[new]
    #[pyo3(signature = (label_names=None, label_path=None, sort_names=false))]
    pub fn new(
        py: Python<'_>,
        label_names: Option<Vec<String>>,
        label_path: Option<String>,
        sort_names: bool,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("LabelToIndex")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("label_names", label_names)?;
        kwargs.set_item("label_path", label_path)?;
        kwargs.set_item("sort_names", sort_names)?;
        let inner = cls.call((), Some(&kwargs))?;
        Ok(Self { inner: inner.unbind() })
    }

    #[getter]
    pub fn label_names(&self, py: Python<'_>) -> PyResult<Vec<String>> {
        let names = self.inner.bind(py).getattr("label_names")?;
        names.extract::<Vec<String>>()
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct Truncate {
    inner: Py<PyAny>,
}

#[pymethods]
impl Truncate {
    #[new]
    pub fn new(py: Python<'_>, max_seq_len: i32) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("Truncate")?;
        let inner = cls.call1((max_seq_len,))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct AddToken {
    inner: Py<PyAny>,
}

#[pymethods]
impl AddToken {
    #[new]
    #[pyo3(signature = (token, begin=true))]
    pub fn new(py: Python<'_>, token: PyObject, begin: bool) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("AddToken")?;
        let inner = cls.call1((token, begin))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct PadTransform {
    inner: Py<PyAny>,
}

#[pymethods]
impl PadTransform {
    #[new]
    pub fn new(py: Python<'_>, max_length: i32, pad_value: i32) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("PadTransform")?;
        let inner = cls.call1((max_length, pad_value))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct StrToIntTransform {
    inner: Py<PyAny>,
}

#[pymethods]
impl StrToIntTransform {
    #[new]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("StrToIntTransform")?;
        let inner = cls.call0()?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct GPT2BPETokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl GPT2BPETokenizer {
    #[new]
    #[pyo3(signature = (encoder_json_path, vocab_bpe_path, return_tokens=false))]
    pub fn new(
        py: Python<'_>,
        encoder_json_path: String,
        vocab_bpe_path: String,
        return_tokens: bool,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("GPT2BPETokenizer")?;
        let inner = cls.call1((encoder_json_path, vocab_bpe_path, return_tokens))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct CharBPETokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl CharBPETokenizer {
    #[new]
    #[pyo3(signature = (bpe_encoder_path, bpe_merges_path, return_tokens=false, unk_token=None, suffix=None, special_tokens=None))]
    pub fn new(
        py: Python<'_>,
        bpe_encoder_path: String,
        bpe_merges_path: String,
        return_tokens: bool,
        unk_token: Option<String>,
        suffix: Option<String>,
        special_tokens: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("CharBPETokenizer")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("bpe_encoder_path", bpe_encoder_path)?;
        kwargs.set_item("bpe_merges_path", bpe_merges_path)?;
        kwargs.set_item("return_tokens", return_tokens)?;
        kwargs.set_item("unk_token", unk_token)?;
        kwargs.set_item("suffix", suffix)?;
        kwargs.set_item("special_tokens", special_tokens)?;
        let inner = cls.call((), Some(&kwargs))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct CLIPTokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl CLIPTokenizer {
    #[new]
    #[pyo3(signature = (merges_path, encoder_json_path=None, num_merges=None, return_tokens=false))]
    pub fn new(
        py: Python<'_>,
        merges_path: String,
        encoder_json_path: Option<String>,
        num_merges: Option<i32>,
        return_tokens: bool,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("CLIPTokenizer")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("merges_path", merges_path)?;
        kwargs.set_item("encoder_json_path", encoder_json_path)?;
        kwargs.set_item("num_merges", num_merges)?;
        kwargs.set_item("return_tokens", return_tokens)?;
        let inner = cls.call((), Some(&kwargs))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct BERTTokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl BERTTokenizer {
    #[new]
    #[pyo3(
        signature = (vocab_path, do_lower_case=true, strip_accents=None, return_tokens=false, never_split=None)
    )]
    pub fn new(
        py: Python<'_>,
        vocab_path: String,
        do_lower_case: bool,
        strip_accents: Option<bool>,
        return_tokens: bool,
        never_split: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("BERTTokenizer")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("vocab_path", vocab_path)?;
        kwargs.set_item("do_lower_case", do_lower_case)?;
        kwargs.set_item("strip_accents", strip_accents)?;
        kwargs.set_item("return_tokens", return_tokens)?;
        kwargs.set_item("never_split", never_split)?;
        let inner = cls.call((), Some(&kwargs))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct RegexTokenizer {
    inner: Py<PyAny>,
}

#[pymethods]
impl RegexTokenizer {
    #[new]
    pub fn new(py: Python<'_>, patterns_list: Vec<(String, String)>) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("RegexTokenizer")?;
        let inner = cls.call1((patterns_list,))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct Sequential {
    inner: Py<PyAny>,
}

#[pymethods]
impl Sequential {
    #[new]
    pub fn new(py: Python<'_>, transforms: Vec<Py<PyAny>>) -> PyResult<Self> {
        // torchtext.transforms.Sequential is a torch.nn.Sequential subclass.
        // It accepts transforms as positional arguments.
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("Sequential")?;
        let args = PyTuple::new(py, transforms)?;
        let inner = cls.call1(args)?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

#[pyclass(unsendable)]
pub struct MaskTransform {
    inner: Py<PyAny>,
}

#[pymethods]
impl MaskTransform {
    #[new]
    #[pyo3(signature = (vocab_len, mask_idx, bos_idx, pad_idx, mask_bos=false, mask_prob=0.15))]
    pub fn new(
        py: Python<'_>,
        vocab_len: i32,
        mask_idx: i32,
        bos_idx: i32,
        pad_idx: i32,
        mask_bos: bool,
        mask_prob: f32,
    ) -> PyResult<Self> {
        let transforms_mod = torchtext_transforms(py)?;
        let cls = transforms_mod.getattr("MaskTransform")?;
        let inner = cls.call1((vocab_len, mask_idx, bos_idx, pad_idx, mask_bos, mask_prob))?;
        Ok(Self { inner: inner.unbind() })
    }

    fn __call__(&self, py: Python<'_>, input: PyObject) -> PyResult<PyObject> {
        let out = self.inner.bind(py).call1((input,))?;
        Ok(out.unbind().into())
    }
}

/// Register torchtext transforms implemented via Python delegation.
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<SentencePieceTokenizer>()?;
    module.add_class::<VocabTransform>()?;
    module.add_class::<ToTensor>()?;
    module.add_class::<LabelToIndex>()?;
    module.add_class::<Truncate>()?;
    module.add_class::<AddToken>()?;
    module.add_class::<PadTransform>()?;
    module.add_class::<StrToIntTransform>()?;
    module.add_class::<GPT2BPETokenizer>()?;
    module.add_class::<CharBPETokenizer>()?;
    module.add_class::<CLIPTokenizer>()?;
    module.add_class::<BERTTokenizer>()?;
    module.add_class::<RegexTokenizer>()?;
    module.add_class::<Sequential>()?;
    module.add_class::<MaskTransform>()?;
    Ok(())
}