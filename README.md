# 🧭 SpaceScanner

**SpaceScanner** es una herramienta de escritorio interactiva diseñada para analizar y visualizar el uso de espacio en disco de forma rápida y eficiente. Permite explorar directorios, identificar archivos o carpetas que consumen almacenamiento excesivo y navegar con fluidez entre rutas, similar a herramientas como *SpaceSniffer* o *WinDirStat*.

El proyecto cuenta con dos implementaciones en este repositorio:
1. **`csharp/`**: Implementación original para Windows desarrollada en **C# (.NET Framework / Windows Forms)**.
2. **`rust/`**: Nueva versión de alto rendimiento y multiplataforma (**Linux y Windows**) desarrollada en **Rust**, con escaneo multi-hilo en paralelo y GUI nativa.

---

## 📊 Comparativa Técnica de Implementaciones

| Característica | Versión C# (.NET / WinForms) | Versión Rust (Multiplataforma) |
| :--- | :--- | :--- |
| **Sistemas Soportados** | Windows (7, 10, 11) | **Linux (Ubuntu, Fedora, etc.) y Windows** |
| **Motor de Escaneo** | Secuencial (un hilo de CPU) | **Multi-hilo en paralelo (aprovecha todos los núcleos)** |
| **Consumo de Memoria RAM** | ~150 MB – 300 MB (CLR + Garbage Collector) | **~10 MB – 25 MB (Gestión estricta sin GC)** |
| **Velocidad de Escaneo** | Estándar para discos locales | **Extrema (optimizado para SSDs / NVMe)** |
| **Dependencias / Runtime** | Requiere .NET Framework 4.7.2 | **Cero dependencias** (binario único nativo) |
| **Distribución** | Ejecutable / Proyecto Visual Studio | **Binarios listos en Releases** (`.exe` y Linux ELF) |

---

## 🚀 Características Principales

- 🔍 **Escaneo rápido con cancelación en tiempo real**: Detén el análisis en cualquier momento sin bloqueos.
- 🌳 **Escaneo profundo configurable por niveles (*Deep Scan*)**: Permite definir cuántos niveles de subdirectorios analizar en profundidad para balancear velocidad y nivel de detalle.
- ⚠️ **Detección visual de elementos excesivos**: Resalta en color aquellos archivos o carpetas que superen un umbral configurable (en KB, MB, GB o TB).
- 💡 **Caché inteligente en memoria**: Almacena rutas ya analizadas (`Dictionary` / `HashMap`) para que regresar a una carpeta anterior sea instantáneo sin reescanear el disco.
- ↩️ **Historial de navegación**: Navegación fluida hacia adelante y atrás mediante pila de historial (`Stack`).
- 🖱️ **Acciones de contexto y exploración**:
  - Doble clic en carpetas para adentrarse en su escaneo.
  - Clic derecho para abrir el recurso en el explorador de archivos nativo (*Explorador de Windows* o *xdg-open* en Linux).
- 💾 **Métricas globales de almacenamiento**: Muestra en tiempo real el espacio libre y total del disco junto al acumulado del directorio.

---

## 📁 Estructura del Repositorio

```text
SpaceScanner/
├── csharp/                      # Proyecto en C# (.NET Framework 4.7.2 / WinForms)
│   ├── SpaceScanner.sln
│   └── SpaceScanner/
│       ├── Form1.cs
│       ├── Form1.Designer.cs
│       └── ...
├── rust/                        # Proyecto en Rust (Motor multi-hilo + GUI multiplataforma)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── README.md
```

---

## ⚙️ Compilación y Uso

### Versión C# (Windows)
1. Abre [`csharp/SpaceScanner.sln`](file:///home/joss/Programs/SpaceScanner/csharp/SpaceScanner.sln) en **Visual Studio**.
2. Selecciona la configuración `Release` o `Debug`.
3. Presiona `Ctrl + F5` para compilar y ejecutar.

### Versión Rust (Linux / Windows)
1. Asegúrate de tener instalado Rust (`rustup` / `cargo`).
2. Entra a la carpeta del proyecto:
   ```bash
   cd rust
   ```
3. Compila y ejecuta en modo optimizado:
   ```bash
   cargo run --release
   ```
*(También puedes descargar directamente los ejecutables precompilados desde la sección de **Releases** de este repositorio).*

---

## 🧠 Detalles Técnicos y Arquitectura

- **Caché de resultados**: Cada carpeta analizada genera una estructura de resultados que se guarda en memoria para evitar llamadas de I/O repetidas al volver atrás.
- **Manejo de Enlaces Simbólicos**: Ambos motores ignoran puntos de reanálisis (*Reparse Points* / Symlinks) para prevenir bucles infinitos en directorios del sistema.
- **Conversión de Unidades**: Soporte para parsear límites de tamaño con sufijos `KB`, `MB`, `GB` y `TB`.
- **Desacoplamiento de Interfaz**: En Rust, el escaneo se procesa en hilos de trabajo independientes y comunica el progreso a la interfaz gráfica mediante canales de mensajería (`mpsc`), garantizando que la ventana nunca se congele.

---

## 🧩 Próximas Mejoras
- [ ] Vista gráfica con mapa de rectángulos proporcionales (*Treemap*) para visualizar visualmente el peso de cada directorio.
- [ ] Filtro por tipos y extensiones de archivo (videos, imágenes, código, comprimidos).
- [ ] Integración de GitHub Actions para compilación automática y despliegue continuo en Releases.

---

## 🪪 Licencia

Este proyecto se distribuye bajo la licencia **MIT**. Puedes usarlo, modificarlo y compartirlo libremente manteniendo la atribución correspondiente.
