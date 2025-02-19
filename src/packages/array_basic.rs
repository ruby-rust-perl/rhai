#![cfg(not(feature = "no_index"))]
#![allow(non_snake_case)]

use crate::engine::OP_EQUALS;
use crate::plugin::*;
use crate::stdlib::{any::TypeId, boxed::Box, cmp::Ordering, mem, string::ToString};
use crate::{def_package, Array, Dynamic, EvalAltResult, FnPtr, NativeCallContext, Position, INT};

def_package!(crate:BasicArrayPackage:"Basic array utilities.", lib, {
    combine_with_exported_module!(lib, "array", array_functions);

    // Register array iterator
    lib.set_iterable::<Array>();
});

#[export_module]
mod array_functions {
    #[rhai_fn(name = "len", get = "len", pure)]
    pub fn len(array: &mut Array) -> INT {
        array.len() as INT
    }
    #[rhai_fn(name = "push", name = "+=")]
    pub fn push(array: &mut Array, item: Dynamic) {
        array.push(item);
    }
    #[rhai_fn(name = "append", name = "+=")]
    pub fn append(array: &mut Array, y: Array) {
        array.extend(y);
    }
    #[rhai_fn(name = "+")]
    pub fn concat(mut array: Array, y: Array) -> Array {
        array.extend(y);
        array
    }
    pub fn insert(array: &mut Array, position: INT, item: Dynamic) {
        if position <= 0 {
            array.insert(0, item);
        } else if (position as usize) >= array.len() {
            push(array, item);
        } else {
            array.insert(position as usize, item);
        }
    }
    #[rhai_fn(return_raw)]
    pub fn pad(
        _ctx: NativeCallContext,
        array: &mut Array,
        len: INT,
        item: Dynamic,
    ) -> Result<(), Box<EvalAltResult>> {
        // Check if array will be over max size limit
        #[cfg(not(feature = "unchecked"))]
        if _ctx.engine().max_array_size() > 0
            && len > 0
            && (len as usize) > _ctx.engine().max_array_size()
        {
            return EvalAltResult::ErrorDataTooLarge("Size of array".to_string(), Position::NONE)
                .into();
        }

        if len > 0 && len as usize > array.len() {
            array.resize(len as usize, item);
        }

        Ok(())
    }
    pub fn pop(array: &mut Array) -> Dynamic {
        array.pop().unwrap_or_else(|| ().into())
    }
    pub fn shift(array: &mut Array) -> Dynamic {
        if array.is_empty() {
            ().into()
        } else {
            array.remove(0)
        }
    }
    pub fn remove(array: &mut Array, len: INT) -> Dynamic {
        if len < 0 || (len as usize) >= array.len() {
            ().into()
        } else {
            array.remove(len as usize)
        }
    }
    pub fn clear(array: &mut Array) {
        array.clear();
    }
    pub fn truncate(array: &mut Array, len: INT) {
        if len >= 0 {
            array.truncate(len as usize);
        } else {
            array.clear();
        }
    }
    pub fn chop(array: &mut Array, len: INT) {
        if len as usize >= array.len() {
        } else if len >= 0 {
            array.drain(0..array.len() - len as usize);
        } else {
            array.clear();
        }
    }
    pub fn reverse(array: &mut Array) {
        array.reverse();
    }
    pub fn splice(array: &mut Array, start: INT, len: INT, replace: Array) {
        let start = if start < 0 {
            0
        } else if start as usize >= array.len() {
            array.len() - 1
        } else {
            start as usize
        };

        let len = if len < 0 {
            0
        } else if len as usize > array.len() - start {
            array.len() - start
        } else {
            len as usize
        };

        array.splice(start..start + len, replace.into_iter());
    }
    pub fn extract(array: &mut Array, start: INT, len: INT) -> Array {
        let start = if start < 0 {
            0
        } else if start as usize >= array.len() {
            array.len() - 1
        } else {
            start as usize
        };

        let len = if len < 0 {
            0
        } else if len as usize > array.len() - start {
            array.len() - start
        } else {
            len as usize
        };

        array[start..start + len].iter().cloned().collect()
    }
    #[rhai_fn(name = "extract")]
    pub fn extract_tail(array: &mut Array, start: INT) -> Array {
        let start = if start < 0 {
            0
        } else if start as usize >= array.len() {
            array.len() - 1
        } else {
            start as usize
        };

        array[start..].iter().cloned().collect()
    }
    #[rhai_fn(name = "split")]
    pub fn split_at(array: &mut Array, start: INT) -> Array {
        if start <= 0 {
            mem::take(array)
        } else if start as usize >= array.len() {
            Default::default()
        } else {
            let mut result: Array = Default::default();
            result.extend(array.drain(start as usize..));
            result
        }
    }
    #[rhai_fn(return_raw, pure)]
    pub fn map(
        ctx: NativeCallContext,
        array: &mut Array,
        mapper: FnPtr,
    ) -> Result<Array, Box<EvalAltResult>> {
        let mut ar = Array::with_capacity(array.len());

        for (i, item) in array.iter().enumerate() {
            ar.push(
                mapper
                    .call_dynamic(&ctx, None, [item.clone()])
                    .or_else(|err| match *err {
                        EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                            if fn_sig.starts_with(mapper.fn_name()) =>
                        {
                            mapper.call_dynamic(&ctx, None, [item.clone(), (i as INT).into()])
                        }
                        _ => Err(err),
                    })
                    .map_err(|err| {
                        Box::new(EvalAltResult::ErrorInFunctionCall(
                            "map".to_string(),
                            ctx.source().unwrap_or("").to_string(),
                            err,
                            Position::NONE,
                        ))
                    })?,
            );
        }

        Ok(ar)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn filter(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<Array, Box<EvalAltResult>> {
        let mut ar = Array::new();

        for (i, item) in array.iter().enumerate() {
            if filter
                .call_dynamic(&ctx, None, [item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "filter".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                ar.push(item.clone());
            }
        }

        Ok(ar)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn contains(
        ctx: NativeCallContext,
        array: &mut Array,
        value: Dynamic,
    ) -> Result<bool, Box<EvalAltResult>> {
        for item in array.iter_mut() {
            if ctx
                .call_fn_dynamic_raw(OP_EQUALS, true, &mut [item, &mut value.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(ref fn_sig, _)
                        if fn_sig.starts_with(OP_EQUALS) =>
                    {
                        if item.type_id() == value.type_id() {
                            // No default when comparing same type
                            Err(err)
                        } else {
                            Ok(Dynamic::FALSE)
                        }
                    }
                    _ => Err(err),
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn index_of(
        ctx: NativeCallContext,
        array: &mut Array,
        value: Dynamic,
    ) -> Result<INT, Box<EvalAltResult>> {
        for (i, item) in array.iter_mut().enumerate() {
            if ctx
                .call_fn_dynamic_raw(OP_EQUALS, true, &mut [item, &mut value.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(ref fn_sig, _)
                        if fn_sig.starts_with(OP_EQUALS) =>
                    {
                        if item.type_id() == value.type_id() {
                            // No default when comparing same type
                            Err(err)
                        } else {
                            Ok(Dynamic::FALSE)
                        }
                    }
                    _ => Err(err),
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(i as INT);
            }
        }

        Ok(-1 as INT)
    }
    #[rhai_fn(name = "index_of", return_raw, pure)]
    pub fn index_of_filter(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<INT, Box<EvalAltResult>> {
        for (i, item) in array.iter().enumerate() {
            if filter
                .call_dynamic(&ctx, None, [item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "index_of".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(i as INT);
            }
        }

        Ok(-1 as INT)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn some(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<bool, Box<EvalAltResult>> {
        for (i, item) in array.iter().enumerate() {
            if filter
                .call_dynamic(&ctx, None, [item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "some".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn all(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<bool, Box<EvalAltResult>> {
        for (i, item) in array.iter().enumerate() {
            if !filter
                .call_dynamic(&ctx, None, [item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "all".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(false);
            }
        }

        Ok(true)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn reduce(
        ctx: NativeCallContext,
        array: &mut Array,
        reducer: FnPtr,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut result = Dynamic::UNIT;

        for (i, item) in array.iter().enumerate() {
            result = reducer
                .call_dynamic(&ctx, None, [result.clone(), item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(reducer.fn_name()) =>
                    {
                        reducer.call_dynamic(&ctx, None, [result, item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "reduce".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?;
        }

        Ok(result)
    }
    #[rhai_fn(name = "reduce", return_raw, pure)]
    pub fn reduce_with_initial(
        ctx: NativeCallContext,
        array: &mut Array,
        reducer: FnPtr,
        initial: Dynamic,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut result = initial;

        for (i, item) in array.iter().enumerate() {
            result = reducer
                .call_dynamic(&ctx, None, [result.clone(), item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(reducer.fn_name()) =>
                    {
                        reducer.call_dynamic(&ctx, None, [result, item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "reduce".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?;
        }

        Ok(result)
    }
    #[rhai_fn(return_raw, pure)]
    pub fn reduce_rev(
        ctx: NativeCallContext,
        array: &mut Array,
        reducer: FnPtr,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut result = Dynamic::UNIT;

        for (i, item) in array.iter().enumerate().rev() {
            result = reducer
                .call_dynamic(&ctx, None, [result.clone(), item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(reducer.fn_name()) =>
                    {
                        reducer.call_dynamic(&ctx, None, [result, item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "reduce_rev".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?;
        }

        Ok(result)
    }
    #[rhai_fn(name = "reduce_rev", return_raw, pure)]
    pub fn reduce_rev_with_initial(
        ctx: NativeCallContext,
        array: &mut Array,
        reducer: FnPtr,
        initial: Dynamic,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut result = initial;

        for (i, item) in array.iter().enumerate().rev() {
            result = reducer
                .call_dynamic(&ctx, None, [result.clone(), item.clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(reducer.fn_name()) =>
                    {
                        reducer.call_dynamic(&ctx, None, [result, item.clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "reduce_rev".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?;
        }

        Ok(result)
    }
    #[rhai_fn(return_raw)]
    pub fn sort(
        ctx: NativeCallContext,
        array: &mut Array,
        comparer: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        array.sort_by(|x, y| {
            comparer
                .call_dynamic(&ctx, None, [x.clone(), y.clone()])
                .ok()
                .and_then(|v| v.as_int().ok())
                .map(|v| {
                    if v > 0 {
                        Ordering::Greater
                    } else if v < 0 {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                })
                .unwrap_or_else(|| {
                    let x_type_id = x.type_id();
                    let y_type_id = y.type_id();

                    if x_type_id > y_type_id {
                        Ordering::Greater
                    } else if x_type_id < y_type_id {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                })
        });

        Ok(())
    }
    #[rhai_fn(return_raw)]
    pub fn drain(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<Array, Box<EvalAltResult>> {
        let mut drained = Array::with_capacity(array.len());

        let mut i = array.len();

        while i > 0 {
            i -= 1;

            if filter
                .call_dynamic(&ctx, None, [array[i].clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [array[i].clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "drain".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                drained.push(array.remove(i));
            }
        }

        Ok(drained)
    }
    #[rhai_fn(name = "drain")]
    pub fn drain_range(array: &mut Array, start: INT, len: INT) -> Array {
        let start = if start < 0 {
            0
        } else if start as usize >= array.len() {
            array.len() - 1
        } else {
            start as usize
        };

        let len = if len < 0 {
            0
        } else if len as usize > array.len() - start {
            array.len() - start
        } else {
            len as usize
        };

        array.drain(start..start + len - 1).collect()
    }
    #[rhai_fn(return_raw)]
    pub fn retain(
        ctx: NativeCallContext,
        array: &mut Array,
        filter: FnPtr,
    ) -> Result<Array, Box<EvalAltResult>> {
        let mut drained = Array::new();

        let mut i = array.len();

        while i > 0 {
            i -= 1;

            if !filter
                .call_dynamic(&ctx, None, [array[i].clone()])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(fn_sig, _)
                        if fn_sig.starts_with(filter.fn_name()) =>
                    {
                        filter.call_dynamic(&ctx, None, [array[i].clone(), (i as INT).into()])
                    }
                    _ => Err(err),
                })
                .map_err(|err| {
                    Box::new(EvalAltResult::ErrorInFunctionCall(
                        "retain".to_string(),
                        ctx.source().unwrap_or("").to_string(),
                        err,
                        Position::NONE,
                    ))
                })?
                .as_bool()
                .unwrap_or(false)
            {
                drained.push(array.remove(i));
            }
        }

        Ok(drained)
    }
    #[rhai_fn(name = "retain")]
    pub fn retain_range(array: &mut Array, start: INT, len: INT) -> Array {
        let start = if start < 0 {
            0
        } else if start as usize >= array.len() {
            array.len() - 1
        } else {
            start as usize
        };

        let len = if len < 0 {
            0
        } else if len as usize > array.len() - start {
            array.len() - start
        } else {
            len as usize
        };

        let mut drained = array.drain(start + len..).collect::<Array>();
        drained.extend(array.drain(..start));

        drained
    }
    #[rhai_fn(name = "==", return_raw, pure)]
    pub fn equals(
        ctx: NativeCallContext,
        array: &mut Array,
        mut array2: Array,
    ) -> Result<bool, Box<EvalAltResult>> {
        if array.len() != array2.len() {
            return Ok(false);
        }
        if array.is_empty() {
            return Ok(true);
        }

        for (a1, a2) in array.iter_mut().zip(array2.iter_mut()) {
            if !ctx
                .call_fn_dynamic_raw(OP_EQUALS, true, &mut [a1, a2])
                .or_else(|err| match *err {
                    EvalAltResult::ErrorFunctionNotFound(ref fn_sig, _)
                        if fn_sig.starts_with(OP_EQUALS) =>
                    {
                        if a1.type_id() == a2.type_id() {
                            // No default when comparing same type
                            Err(err)
                        } else {
                            Ok(Dynamic::FALSE)
                        }
                    }
                    _ => Err(err),
                })?
                .as_bool()
                .unwrap_or(false)
            {
                return Ok(false);
            }
        }

        Ok(true)
    }
    #[rhai_fn(name = "!=", return_raw, pure)]
    pub fn not_equals(
        ctx: NativeCallContext,
        array: &mut Array,
        array2: Array,
    ) -> Result<bool, Box<EvalAltResult>> {
        equals(ctx, array, array2).map(|r| !r)
    }
}
