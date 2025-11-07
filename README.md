# 🧭 SpaceScanner

**SpaceScanner** es una aplicación de escritorio en **C# (Windows Forms)** que analiza el uso de espacio en disco de forma visual e interactiva.  
Permite explorar carpetas, identificar archivos o directorios que ocupan demasiado espacio y navegar entre resultados fácilmente, similar a herramientas como *SpaceSniffer* o *WinDirStat*.

---

## 🚀 Características principales

- 🔍 **Escaneo rápido y seguro** de carpetas, con cancelación en tiempo real.  
- 💾 **Visualización del espacio utilizado** por cada archivo y subcarpeta.  
- ⚠️ **Detección de elementos “excesivos”** según un tamaño límite configurable.  
- 💡 **Caché inteligente**: los resultados escaneados se guardan en memoria, evitando volver a calcular rutas previamente analizadas.  
- ↩️ **Navegación entre rutas**: puedes avanzar dentro de una carpeta o retroceder a la anterior sin perder los datos ya cargados.    
- 🖱️ **Interacción con clic derecho o doble clic**:
  - Abrir carpeta o archivo en el explorador de Windows.
  - Escanear una subcarpeta directamente.

---

## 🧩 Tecnologías utilizadas

- **Lenguaje:** C#  
- **Framework:** .NET Framework / Windows Forms  
- **Librerías estándar:**  
  - `System.IO` — para manejo de archivos y directorios  
  - `System.Threading.Tasks` — para escaneo asincrónico  
  - `System.Windows.Forms` — interfaz gráfica  
  - `System.Drawing` — para renderizado visual  

---

## ⚙️ Instalación y uso

1. Clona el repositorio:
   ```bash
   git clone https://github.com/tuusuario/SpaceScanner.git
2. Abre el proyecto en Visual Studio.

3. Compila y ejecuta (Ctrl + F5).

4. En la interfaz principal:
- Introduce una ruta válida en el campo Ruta (txtPath).
- Define el tamaño límite (por ejemplo, 500 MB, 2 GB, etc.).
- Presiona Escanear para iniciar el análisis.

5. Puedes detener el proceso con Detener o navegar con los botones de Atrás.

---

## 🧠 Detalles técnicos
- Los resultados se almacenan en un Dictionary<string, ScanResult> (scanCache) para evitar recálculos.

- Se mantiene un historial de navegación (Stack<string> navigationHistory) que permite moverse entre rutas escaneadas.

- Cada escaneo calcula:
  Tamaño total de la carpeta
  Espacio libre y total del disco
  Tamaño individual de archivos y subcarpetas

- La comparación con el límite de tamaño (txtSize) se hace en MB, con soporte para unidades: KB, MB, GB, TB.

---

## 🧩 Próximas mejoras
1. **Modo gráfico** para visualizar de manera más amigable los directorios escaneados.
