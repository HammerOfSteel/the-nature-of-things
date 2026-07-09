# Project Overview: Emergent Slice-of-Life Sim

## 1. Core Concept
A cozy, bohemian slice-of-life simulation set in a procedurally generated small town inspired by the South Wales valleys. The game leans heavily into autonomous simulation rather than direct player control. The visual style targets a top-down (Zelda: Link to the Past) or 2D side-view pixel art aesthetic, generated procedurally via math and shaders rather than hand-painted assets.

## 2. The "Behavioral Automata" Engine (Key Innovation)
Instead of traditional Finite State Machines (FSM) or Behavior Trees, character (actor) AI is driven by abstracted Neural Cellular Automata (NCA) / Conway-style logic.
*   **The Concept:** A character's internal state (stress, hunger, social need, local Welsh valley cultural ties) and their environment are represented as interconnected nodes (a graph or grid).
*   **Emergence:** On every simulation tick, states "bleed" or propagate to neighboring nodes based on mathematical rules. If "work stress" reaches a threshold, it cascades into the "social" node, organically changing the actor's next choice without hardcoded if/else branches.
*   **Outcome:** Actors live unique, branching lives governed by systemic math, resulting in highly organic and unpredictable town dynamics.

## 3. Technical Constraints & Architecture
*   **Environment:** 100% pure-code repository entirely driven from VS Code. No proprietary visual engine editors (no Unity/Godot GUI).
*   **Hardware Target:** Optimized for local development and fast compilation (e.g., Apple Silicon M4 environments).
*   **Decoupling:** The architecture must strictly separate the simulation state (the automated world) from the visual rendering step. The simulation must be able to run headless.
*   **Framework Options:** To be determined based on ECS (Entity-Component-System) compatibility (e.g., Bevy, Raylib, or a custom Typescript/Phaser loop).
*   **Asset Generation:** Assets, terrains, and structures will be generated via code (Noise functions, WFC, SDFs) rather than loaded from static sprite sheets.

## 4. AI Directives
Based on the above constraints and vision, please generate the following scaffolding documents in this repository:

1.  **`gdd.md` (Game Design Document):** Flesh out the core mechanics of the Behavioral Automata. Define the exact "nodes" that will make up a character's state grid and how they influence the cozy, slice-of-life gameplay. Incorporate thematic elements of Welsh valley nature and community.
2.  **`architecture.md`:** Propose a pure-code, data-driven architecture (preferably ECS) that isolates the math-heavy simulation from the rendering layer.
3.  **`readme.md`:** A standard open-source style README summarizing the project, the tech stack, and how to run the headless simulation loop.
4.  **`todo.md`:** A prioritized, step-by-step implementation plan. Phase 1 must focus purely on terminal-based output of the behavioral automata before any visual framework is attached.
