using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SpaceScanner
{
    public partial class Form1 : Form
    {
        private CancellationTokenSource cts;
        private Dictionary<string, ScanResult> scanCache = new Dictionary<string, ScanResult>();
        private Stack<string> navigationHistory = new Stack<string>();
        private string currentPath = "";

        public Form1()
        {
            InitializeComponent();
            lblTotalSize.Text = "";
            lblFreeSize.Text = "";
            lblGlobalSize.Text = "";
            btnStop.Enabled = false;
            btnScan.Enabled = true;
            txtSize.ReadOnly = false;
            txtPath.ReadOnly = false;
            btnAtras.Enabled = false;
            txtCacheIndicator.Visible = false;
            graphPanel.Visible = false;

            txtSize.Text = "5000 MB";
            txtPath.Text = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            //asociar el menú contextual al DataGridView
            gridResults.ContextMenuStrip = contextMenuGrid;
            gridResults.MouseDown += gridResults_MouseDown;
        }

        private void btnScan_Click(object sender, EventArgs e)
        {
            EscanearRuta(false);
        }

        private void chkDeepScan_CheckedChanged(object sender, EventArgs e)
        {
            numLevels.Enabled = chkDeepScan.Checked;
            lblLevels.Enabled = chkDeepScan.Checked;
        }

        private async void EscanearRuta(bool takeHistorial)
        {
            string rootPath = txtPath.Text.Trim();
            currentPath = rootPath;
            txtCacheIndicator.Visible = false;
            if (File.Exists(rootPath))
            {
                MessageBox.Show("No se puede escanear un archivo. Por favor ingrese una ruta de carpeta válida.", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return;
            }
            else if (string.IsNullOrEmpty(rootPath) || !Directory.Exists(rootPath) || rootPath.Contains(".sys"))
            {
                MessageBox.Show("Por favor ingrese una ruta válida.", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return;
            }

            //obtener el límite de tamaño desde txtSize (en MB)
            double limiteExcesivoMB = ParseSize(txtSize.Text);

            //verificar si ya existe en caché
            if (scanCache.ContainsKey(rootPath) && takeHistorial)
            {
                var cached = scanCache[rootPath];
                gridResults.Rows.Clear();
                gridResults.Columns.Clear();

                gridResults.Columns.Add("ColName", "Nombre");
                gridResults.Columns.Add("ColPath", "Ruta");
                gridResults.Columns.Add("ColType", "Tipo");
                gridResults.Columns.Add("ColSize", "Tamaño (MB)");
                gridResults.Columns.Add("ColStatus", "Estado");
                gridResults.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;

                foreach (var item in cached.Items)
                {
                    string estado = item.SizeMB > limiteExcesivoMB ? "⚠️ Excesivo" : "✅ Normal";
                    int rowIndex = gridResults.Rows.Add(item.Name, item.Path, item.Type, item.SizeMB.ToString("N2"), estado);
                    gridResults.Rows[rowIndex].DefaultCellStyle.BackColor =
                        item.Type == "Carpeta" ? Color.LightYellow : Color.Azure;

                    if (item.SizeMB > limiteExcesivoMB)
                    {
                        gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Red;
                        gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Bold);
                    }
                    else
                    {
                        gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Green;
                        gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Regular);
                    }
                }

                lblTotalSize.Text = $"Total ruta: {cached.TotalSizeMB:N2} MB";
                lblFreeSize.Text = $"Libre: {cached.FreeSpaceMB:N2} MB";
                lblGlobalSize.Text = $"Total disco: {cached.DiskTotalMB:N2} MB";

                btnStop.Enabled = false;
                btnScan.Enabled = true;
                btnAtras.Enabled = navigationHistory.Count > 0;
                txtSize.ReadOnly = false;
                txtPath.ReadOnly = false;
                chkDeepScan.Enabled = true;
                numLevels.Enabled = chkDeepScan.Checked;

                txtCacheIndicator.Visible = true;
                //MessageBox.Show("Datos cargados desde memoria.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                return;
            }

            lblTotalSize.Text = "Calculando...";
            lblFreeSize.Text = "";
            lblGlobalSize.Text = "";
            btnStop.Enabled = true;
            btnScan.Enabled = false;
            btnAtras.Enabled = false;
            txtSize.ReadOnly = true;
            txtPath.ReadOnly = true;
            chkDeepScan.Enabled = false;
            numLevels.Enabled = false;
            takeHistorial = false;

            int maxDepth = chkDeepScan.Checked ? (int)numLevels.Value : 1;

            //cancelar cualquier escaneo previo
            cts?.Cancel();
            cts = new CancellationTokenSource();
            var token = cts.Token;

            string driveRoot = Path.GetPathRoot(rootPath);
            double driveFreeMB = 0;
            double driveTotalMB = 0;

            try
            {
                DriveInfo drive = new DriveInfo(driveRoot);
                driveFreeMB = drive.AvailableFreeSpace / (1024.0 * 1024);
                driveTotalMB = drive.TotalSize / (1024.0 * 1024);
                lblFreeSize.Text = $"Libre: {driveFreeMB:N2} MB";
                lblGlobalSize.Text = $"Total disco: {driveTotalMB:N2} MB";
            }
            catch
            {
                lblFreeSize.Text = "Libre: N/D";
                lblGlobalSize.Text = "Total disco: N/D";
            }

            gridResults.Rows.Clear();
            gridResults.Columns.Clear();
            gridResults.Columns.Add("ColName", "Nombre");
            gridResults.Columns.Add("ColPath", "Ruta");
            gridResults.Columns.Add("ColType", "Tipo");
            gridResults.Columns.Add("ColSize", "Tamaño (MB)");
            gridResults.Columns.Add("ColStatus", "Estado"); // columna de estado
            gridResults.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;

            long totalSize = 0;

            try
            {
                await Task.Run(() =>
                {
                    totalSize = ScanDirectoryNode(rootPath, 1, maxDepth, limiteExcesivoMB, driveFreeMB, driveTotalMB, token, true);
                });

                if (!token.IsCancellationRequested)
                {
                    btnStop.Enabled = false;
                    btnScan.Enabled = true;
                    btnAtras.Enabled = navigationHistory.Count > 0;
                    txtSize.ReadOnly = false;
                    txtPath.ReadOnly = false;
                    chkDeepScan.Enabled = true;
                    numLevels.Enabled = chkDeepScan.Checked;
                    lblTotalSize.Text = $"Total ruta: {(totalSize / (1024.0 * 1024)).ToString("N2")} MB";
                    MessageBox.Show("Escaneo terminado.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                }
                else
                {
                    btnStop.Enabled = false;
                    btnScan.Enabled = true;
                    btnAtras.Enabled = navigationHistory.Count > 0;
                    txtSize.ReadOnly = false;
                    txtPath.ReadOnly = false;
                    chkDeepScan.Enabled = true;
                    numLevels.Enabled = chkDeepScan.Checked;
                    gridResults.Rows.Clear();
                    gridResults.Columns.Clear();
                    lblTotalSize.Text = "Escaneo cancelado";
                    MessageBox.Show("Escaneo cancelado.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                }
            }
            catch
            {
                btnStop.Enabled = false;
                btnScan.Enabled = true;
                btnAtras.Enabled = navigationHistory.Count > 0;
                txtSize.ReadOnly = false;
                txtPath.ReadOnly = false;
                chkDeepScan.Enabled = true;
                numLevels.Enabled = chkDeepScan.Checked;
                lblTotalSize.Text = "Error durante el escaneo";
            }
        }

        private long ScanDirectoryNode(string dirPath, int currentDepth, int maxDepth, double limiteExcesivoMB, double freeMB, double diskMB, CancellationToken token, bool isRoot)
        {
            if (token.IsCancellationRequested) return 0;

            long totalDirSize = 0;
            var items = new List<ScanItem>();

            // Obtener archivos directos
            string[] files = Array.Empty<string>();
            try { files = Directory.GetFiles(dirPath); } catch { }

            foreach (var file in files)
            {
                if (token.IsCancellationRequested) return totalDirSize;
                long fSize = 0;
                try { fSize = new FileInfo(file).Length; } catch { }

                totalDirSize += fSize;
                double fSizeMB = fSize / (1024.0 * 1024);
                string fEstado = fSizeMB > limiteExcesivoMB ? "⚠️ Excesivo" : "✅ Normal";

                items.Add(new ScanItem
                {
                    Name = Path.GetFileName(file),
                    Path = file,
                    Type = "Archivo",
                    SizeMB = fSizeMB,
                    Estado = fEstado
                });
            }

            // Si es la raíz, agregar archivos al grid al inicio
            if (isRoot && files.Length > 0 && !token.IsCancellationRequested)
            {
                this.BeginInvoke((Action)(() =>
                {
                    if (token.IsCancellationRequested) return;
                    foreach (var fileItem in items.Where(i => i.Type == "Archivo"))
                    {
                        int rowIndex = gridResults.Rows.Add(
                            fileItem.Name,
                            fileItem.Path,
                            fileItem.Type,
                            fileItem.SizeMB.ToString("N2"),
                            fileItem.Estado
                        );
                        gridResults.Rows[rowIndex].DefaultCellStyle.BackColor = Color.Azure;

                        if (fileItem.SizeMB > limiteExcesivoMB)
                        {
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Red;
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Bold);
                        }
                        else
                        {
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Green;
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Regular);
                        }
                    }
                }));
            }

            // Obtener subdirectorios directos
            string[] subDirs = Array.Empty<string>();
            try { subDirs = Directory.GetDirectories(dirPath); } catch { }

            foreach (var subDir in subDirs)
            {
                if (token.IsCancellationRequested) return totalDirSize;

                // Omitir enlaces simbólicos y puntos de reanálisis para evitar ciclos infinitos (ej. Application Data)
                try
                {
                    var di = new DirectoryInfo(subDir);
                    if ((di.Attributes & FileAttributes.ReparsePoint) == FileAttributes.ReparsePoint)
                    {
                        continue;
                    }
                }
                catch { }

                long subDirSize = 0;
                if (currentDepth < maxDepth)
                {
                    // Escaneo recursivo y población de caché para niveles inferiores
                    subDirSize = ScanDirectoryNode(subDir, currentDepth + 1, maxDepth, limiteExcesivoMB, freeMB, diskMB, token, false);
                }
                else
                {
                    // Nivel límite alcanzado: cálculo recursivo del tamaño sin expandir caché de niveles más profundos
                    int count = 0;
                    subDirSize = GetDirectorySizeSafe(subDir, ref count, token);
                }

                totalDirSize += subDirSize;
                double subSizeMB = subDirSize / (1024.0 * 1024);
                string dEstado = subSizeMB > limiteExcesivoMB ? "⚠️ Excesivo" : "✅ Normal";

                items.Add(new ScanItem
                {
                    Name = Path.GetFileName(subDir),
                    Path = subDir,
                    Type = "Carpeta",
                    SizeMB = subSizeMB,
                    Estado = dEstado
                });

                // Si es la ruta raíz mostrada, actualizar el grid en tiempo real a medida que cada carpeta termina su cálculo
                if (isRoot && !token.IsCancellationRequested)
                {
                    this.BeginInvoke((Action)(() =>
                    {
                        if (token.IsCancellationRequested) return;
                        int rowIndex = gridResults.Rows.Add(
                            Path.GetFileName(subDir),
                            subDir,
                            "Carpeta",
                            subSizeMB.ToString("N2"),
                            dEstado
                        );
                        gridResults.Rows[rowIndex].DefaultCellStyle.BackColor = Color.LightYellow;

                        if (subSizeMB > limiteExcesivoMB)
                        {
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Red;
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Bold);
                        }
                        else
                        {
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Green;
                            gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Regular);
                        }

                        lblTotalSize.Text = $"Total acumulado: {(totalDirSize / (1024.0 * 1024)).ToString("N2")} MB";
                    }));
                }
            }

            // Guardar en caché el resultado completo de este directorio
            if (!token.IsCancellationRequested)
            {
                lock (scanCache)
                {
                    scanCache[dirPath] = new ScanResult
                    {
                        Items = items,
                        TotalSizeMB = totalDirSize / (1024.0 * 1024),
                        FreeSpaceMB = freeMB,
                        DiskTotalMB = diskMB
                    };
                }
            }

            return totalDirSize;
        }

        private double ParseSize(string text)
        {
            // Diccionario de multiplicadores (en MB)
            Dictionary<string, double> sizeMultipliers = new Dictionary<string, double>(StringComparer.OrdinalIgnoreCase)
            {
                { "KB", 1.0 / 1024 },
                { "MB", 1 },
                { "GB", 1024 },
                { "TB", 1024 * 1024 }
            };

            if (string.IsNullOrWhiteSpace(text))
                return 5000; // valor por defecto

            string numero = new string(text.Where(c => char.IsDigit(c) || c == '.').ToArray());
            if (!double.TryParse(numero, out double valor))
                return 5000;

            foreach (var kvp in sizeMultipliers)
            {
                if (text.ToUpper().Contains(kvp.Key))
                {
                    return valor * kvp.Value;
                }
            }

            return valor;
        }

        private long GetDirectorySizeSafe(string path, ref int count, CancellationToken token)
        {
            long size = 0;
            try
            {
                string[] files = Array.Empty<string>();
                try { files = Directory.GetFiles(path); } catch { }
                foreach (string file in files)
                {
                    if (token.IsCancellationRequested) return size;
                    try { size += new FileInfo(file).Length; } catch { }
                }

                string[] dirs = Array.Empty<string>();
                try { dirs = Directory.GetDirectories(path); } catch { }
                foreach (string dir in dirs)
                {
                    if (token.IsCancellationRequested) return size;
                    try
                    {
                        var di = new DirectoryInfo(dir);
                        if ((di.Attributes & FileAttributes.ReparsePoint) == FileAttributes.ReparsePoint)
                            continue;
                    }
                    catch { }
                    size += GetDirectorySizeSafe(dir, ref count, token);
                }
            }
            catch { }

            count++;
            return size;
        }

        private void txtPath_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.KeyCode == Keys.Enter)
            {
                e.SuppressKeyPress = true;
                EscanearRuta(true);
            }
        }

        private void btnStop_Click(object sender, EventArgs e)
        {
            DialogResult result = MessageBox.Show("¿Está seguro que desea cancelar el escaneo?", "Mensaje", MessageBoxButtons.YesNo, MessageBoxIcon.Question);

            if (result == DialogResult.Yes)
            {
                cts?.Cancel();
            }
        }

        private void gridResults_MouseDown(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Right)
            {
                var hit = gridResults.HitTest(e.X, e.Y);
                if (hit.RowIndex >= 0)
                {
                    gridResults.ClearSelection();
                    gridResults.Rows[hit.RowIndex].Selected = true;
                }
            }
        }

        private void abrirRecursoToolStripMenuItem_Click(object sender, EventArgs e)
        {
            if (gridResults.SelectedRows.Count > 0)
            {
                string ruta = gridResults.SelectedRows[0].Cells["ColPath"].Value.ToString();
                if (Directory.Exists(ruta) || File.Exists(ruta))
                {
                    System.Diagnostics.Process.Start("explorer.exe", ruta);
                }
            }
        }

        private void escanearRutaToolStripMenuItem_Click(object sender, EventArgs e)
        {
            if (gridResults.SelectedRows.Count > 0)
            {
                string tipo = gridResults.SelectedRows[0].Cells["ColType"].Value?.ToString();
                if (tipo != "Carpeta") return;

                navigationHistory.Push(currentPath); // guardar la ruta actual
                string ruta = gridResults.SelectedRows[0].Cells["ColPath"].Value.ToString();
                txtPath.Text = ruta;
                EscanearRuta(true);
            }
        }

        private void escanearRutaToolStripMenuItem_DoubleClick(object sender, DataGridViewCellEventArgs e)
        {
            if (e.RowIndex < 0) return;

            string tipo = gridResults.Rows[e.RowIndex].Cells["ColType"].Value?.ToString();
            if (tipo != "Carpeta") return;

            // Obtener la ruta desde la fila doble clickeada
            string ruta = gridResults.Rows[e.RowIndex].Cells["ColPath"].Value?.ToString();
            if (string.IsNullOrEmpty(ruta)) return;

            navigationHistory.Push(currentPath);
            txtPath.Text = ruta;
            EscanearRuta(true);
        }

        private void btnBack_Click(object sender, EventArgs e)
        {
            if (navigationHistory.Count > 0)
            {
                string previous = navigationHistory.Pop();
                if (!string.IsNullOrEmpty(previous))
                {
                    txtPath.Text = previous;
                    EscanearRuta(true);
                }
            }
        }

        private void vistaDetalladaToolStripMenuItem_Click(object sender, EventArgs e)
        {
            graphPanel.Visible = false;
        }

        private void vistaGraficaToolStripMenuItem_Click(object sender, EventArgs e)
        {
            graphPanel.Visible = true;
        }

        /*
         * 
         * CLASES
         * 
         */

        class ScanResult
        {
            public List<ScanItem> Items { get; set; }
            public double TotalSizeMB { get; set; }
            public double FreeSpaceMB { get; set; }
            public double DiskTotalMB { get; set; }
        }

        class ScanItem
        {
            public string Name { get; set; }
            public string Path { get; set; }
            public string Type { get; set; }
            public double SizeMB { get; set; }
            public string Estado { get; set; }
        }
    }
}
