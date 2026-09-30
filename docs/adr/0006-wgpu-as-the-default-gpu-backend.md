# 0006. wgpu as the default GPU backend

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

Tantu needs a GPU renderer that works on Windows, macOS and Linux, and a fallback for machines
where the GPU path fails: VMs, RDP/Citrix sessions and old GPUs. Because the renderer contract is
the Scene (ADR 0003), backends can be swapped without touching UI code.

## Decision

- **`tantu-render-wgpu`** is the default backend. It uses wgpu (Vulkan, Metal, DX12 and GL) with
  our own pipelines for the Scene commands. The pipeline design goes in the renderer spec.
- **`tantu-render-soft`** (tiny-skia) is the CPU fallback and the reference for golden images in
  CI.
- **Vello** will be evaluated for path-heavy content (charts, vector graphics) once it is stable
  enough. It would be added as another backend or as a path for custom commands, not as a
  replacement for the Scene contract.
- Only `tantu-render-*` crates may depend on wgpu or tiny-skia.

## Consequences

- One GPU code path covers all three desktop OSes, including older machines through the GL
  backend.
- wgpu is a large dependency (compile time, binary size). The facade's `wgpu` feature is on by
  default but can be turned off for software-only builds.
- wgpu may require a newer compiler than our MSRV (Rust 1.85). When it does, the MSRV rises with
  it.
- The wgpu and software backends must produce matching output. Small anti-aliasing differences are
  expected, so golden comparisons need a tolerance. The Phase 1 milestone tests exactly this.

## Alternatives considered

- **Vello as the default now.** It relies on GPU compute, which not all of our targets support
  (older GL drivers, some VMs), and it is still evolving. Worth revisiting.
- **Skia through bindings.** Mature, but a large C++ build dependency.
- **One backend per native API** (Metal, DX12, Vulkan). Most control, but three times the renderer
  work. wgpu already abstracts over them.
