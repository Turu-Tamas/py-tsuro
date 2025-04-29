use itertools::Itertools;
use pyo3::{prelude::*, types::PyBytes};
use std::default::Default;
use std::fmt::Debug;
use bincode::config;
use bincode::serde::{encode_to_vec, decode_from_slice};

use crate::*;

impl Default for BoardGraph {
    fn default() -> Self {
        Self {
            vertices: [None; 168],
            adjacency_list: [const { vec![] }; 168],
        }
    }
}

macro_rules! impl_python_other_methods {
    ($type:ty) => {
        #[pymethods]
        impl $type {
            pub fn __str__(&self) -> String {
                format!("{:?}", self)
            }

            pub fn __repr__(&self) -> String {
                self.__str__()
            }

            pub fn __getstate__(&self, py: Python<'_>) -> PyObject {
                let serialized = encode_to_vec(self, config::standard()).unwrap();
                PyBytes::new(py, &serialized).into()
            }

            pub fn __setstate__(
                &mut self,
                py: Python<'_>,
                state: PyObject,
            ) -> PyResult<()> {
                let bytes = state.extract::<&[u8]>(py)?;
                (*self, _) = decode_from_slice(bytes, config::standard()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
                })?;
                Ok(())
            }

            pub fn __eq__(&self, other: &Self) -> bool {
                self == other
            }
        }
    };
}

macro_rules! impl_python_new {
    ($type:ty) => {
        #[pymethods]
        impl $type {
            #[new]
            #[allow(clippy::new_ret_no_self)]
            fn py_new() -> PyResult<Self> {
                Ok(Default::default())
            }
        }
    };
}

macro_rules! impl_python_methods {
    ($type:ty) => {
        impl_python_other_methods!($type);
        impl_python_new!($type);
    };
    ($type:ty, skip_new) => {
        impl_python_other_methods!($type);
    };
}

impl Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chars = self
            .connections
            .iter()
            .map(|num| (b'1' + *num as u8) as char)
            .collect_vec();
        let mut out = String::new();
        for i in 0..4 {
            out.push(chars[i * 2]);
            out.push(chars[i * 2 + 1]);
            if i != 3 {
                out.push('-');
            }
        }
        f.write_str(out.as_str())?;
        Ok(())
    }
}

#[pymethods]
impl Board {
    #[getter]
    pub fn markers(&self) -> Vec<Option<MarkerPosition>> {
        self.markers.iter().map(|m| m.map(|m| m.position)).collect()
    }
}

impl_python_methods!(Board);
impl_python_methods!(Tile);
impl_python_methods!(Phase);
impl_python_methods!(BoardGraph);
impl_python_methods!(Marker);
impl_python_methods!(View);
impl_python_methods!(TsuroEnv, skip_new); // new is already implemented here
impl_python_methods!(MarkerPosition);
impl_python_methods!(EnvReturn);
