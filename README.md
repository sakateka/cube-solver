# 🎯 Cube Solver

A cross-platform 3D Rubik's Cube solver built with **Bevy** and **Rust**. Experience the magic of solving a 3x3x3 cube with beautiful 3D graphics, intuitive touch controls, and a powerful solving engine.

## 🏗️ Architecture Overview

### Core Components

The project is organized as a Rust workspace with three main crates:

- **`cube_solver`** - Core game engine and logic
- **`cube_android`** - Android-specific bindings and configuration
- **`cube_ios`** - iOS-specific bindings and configuration

### 📁 Codebase Structure

```
cube-solver/
├── cube_solver/                 # Core game engine
│   ├── src/
│   │   ├── app.rs              # Main application setup and systems
│   │   ├── cube.rs             # 3D cube creation and management
│   │   ├── cube_moves.rs       # Move validation and execution
│   │   ├── layer_components.rs # Layer-based cube architecture
│   │   ├── layer_rotation.rs   # Smooth rotation animations
│   │   ├── solver_integration.rs # min2phase solver integration
│   │   ├── selection.rs        # Touch/click selection system
│   │   ├── camera.rs           # 3D camera and lighting setup
│   │   ├── colors.rs           # Color management and materials
│   │   ├── input.rs            # Input handling (touch/mouse)
│   │   ├── ray_caster.rs       # 3D ray casting for selection
│   │   └── ui/                 # User interface components
│   │       ├── color_panel.rs  # Color selection interface
│   │       ├── solve.rs        # Solve button and solution display
│   │       ├── navigation.rs   # Navigation controls
│   │       └── rotations_panel.rs # Rotation controls
│   └── assets/                 # Game assets (fonts, textures)
├── cube_android/               # Android platform support
├── cube_ios/                   # iOS platform support
└── scripts/                    # Build and deployment scripts
    └── run_android_emulator.sh # Android emulator setup
```

### Key Design Patterns

- **Entity Component System (ECS)**: Leverages Bevy's ECS for clean, performant code
- **Layer-based Cube Representation**: Sophisticated 3D cube with 9 independent layers
- **Event-driven Architecture**: Clean separation of concerns with custom events
- **Resource Management**: Centralized color and solver state management

## 🚀 Getting Started

### Building from Source
```bash
git clone https://github.com/sakateka/cube-solver.git
cd cube-solver

# for android
cd cube_android
x build --device adb:$YOUR_PHONE_ID
# or
x build --platform android --arch arm64
```

**Note**: Currently having trouble building for iOS devices under Linux.

### Basic Controls

- **Rotate Cube**: Click and drag to rotate the entire cube
- **Select Face**: Click on any face to color it
- **Change Color**: Use the color panel to coose another color
- **Solve**: Click the solve button to get an optimal solution
- **Navigate**: Use navigation buttons to step through solution moves


## 📱 Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **Android** | ✅ Full Support | Touch-optimized interface |
| **iOS** | ✅ Full Support | Native iOS integration |

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Bevy Engine](https://bevyengine.org/)** - For the amazing game engine
- **[min2phase_rust](https://github.com/cs0x7f/min2phase_rust)** - For the optimal cube solving algorithm
- **[xbuild](https://github.com/rust-mobile/xbuild)** - For seamless cross-platform compilation
- **Rust Community** - For the excellent ecosystem and tools

---

**Happy Solving! 🎯**

*❤️ Built with Bevy*
