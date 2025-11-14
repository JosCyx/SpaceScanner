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

        private async void btnScan_Click(object sender, EventArgs e)
        {
            EscanearRuta(false);
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
                    int rowIndex = gridResults.Rows.Add(item.Name, item.Path, item.Type, item.SizeMB.ToString("N2"), item.Estado);
                    gridResults.Rows[rowIndex].DefaultCellStyle.BackColor =
                        item.Type == "Carpeta" ? Color.LightYellow : Color.Azure;
                }

                lblTotalSize.Text = $"Total ruta: {cached.TotalSizeMB:N2} MB";
                lblFreeSize.Text = $"Libre: {cached.FreeSpaceMB:N2} MB";
                lblGlobalSize.Text = $"Total disco: {cached.DiskTotalMB:N2} MB";

                txtCacheIndicator.Visible = true;
                //MessageBox.Show("Datos cargados desde memoria.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                return;
            }

            lblTotalSize.Text = "Calculando...";
            lblFreeSize.Text = "";
            lblGlobalSize.Text = "";
            btnStop.Enabled = true;
            btnScan.Enabled = false;
            txtSize.ReadOnly = true;
            txtPath.ReadOnly = true;
            takeHistorial = false;

            //obtener el límite de tamaño desde txtSize (en MB)
            double limiteExcesivoMB = ParseSize(txtSize.Text);

            //cancelar cualquier escaneo previo
            cts?.Cancel();
            cts = new CancellationTokenSource();
            var token = cts.Token;

            string driveRoot = Path.GetPathRoot(rootPath);
            DriveInfo drive = null;

            try
            {
                drive = new DriveInfo(driveRoot);
                lblFreeSize.Text = $"Libre: {(drive.AvailableFreeSpace / (1024.0 * 1024)).ToString("N2")} MB";
                lblGlobalSize.Text = $"Total disco: {(drive.TotalSize / (1024.0 * 1024)).ToString("N2")} MB";
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

            string[] directories = Array.Empty<string>();
            string[] files = Array.Empty<string>();

            try { directories = Directory.GetDirectories(rootPath); } catch { }
            try { files = Directory.GetFiles(rootPath); } catch { }

            long totalSize = 0;

            try
            {
                await Task.Run(() =>
                {
                    int count = 0;

                    //directorios
                    foreach (var dir in directories)
                    {
                        if (token.IsCancellationRequested) return;
                        long size = GetDirectorySizeSafe(dir, ref count, token);
                        totalSize += size;

                        this.Invoke((Action)(() =>
                        {
                            double sizeMB = size / (1024.0 * 1024);
                            string estadoCarpeta = sizeMB > limiteExcesivoMB ? "⚠️ Excesivo" : "✅ Normal";

                            int rowIndex = gridResults.Rows.Add(
                                Path.GetFileName(dir),
                                dir,
                                "Carpeta",
                                sizeMB.ToString("N2"),
                                estadoCarpeta
                            );
                            gridResults.Rows[rowIndex].DefaultCellStyle.BackColor = Color.LightYellow;

                            if (sizeMB > limiteExcesivoMB)
                            {
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Red;
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Bold);
                            }
                            else
                            {
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Green;
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Regular);
                            }
                        }));
                    }

                    //archivos
                    foreach (var file in files)
                    {
                        if (token.IsCancellationRequested) return;
                        long size = 0;
                        try { size = new FileInfo(file).Length; } catch { }

                        totalSize += size;
                        count++;

                        this.Invoke((Action)(() =>
                        {
                            double sizeMB = size / (1024.0 * 1024);
                            string estadoArchivo = sizeMB > limiteExcesivoMB ? "⚠️ Excesivo" : "✅ Normal";

                            int rowIndex = gridResults.Rows.Add(
                                Path.GetFileName(file),
                                file,
                                "Archivo",
                                sizeMB.ToString("N2"),
                                estadoArchivo
                            );
                            gridResults.Rows[rowIndex].DefaultCellStyle.BackColor = Color.Azure;

                            if (sizeMB > limiteExcesivoMB)
                            {
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Red;
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Bold);
                            }
                            else
                            {
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.ForeColor = Color.Green;
                                gridResults.Rows[rowIndex].Cells["ColStatus"].Style.Font = new Font(gridResults.Font, FontStyle.Regular);
                            }
                        }));
                    }
                });

                var items = new List<ScanItem>();

                foreach (DataGridViewRow row in gridResults.Rows)
                {
                    if (!row.IsNewRow)
                    {
                        items.Add(new ScanItem
                        {
                            Name = row.Cells["ColName"].Value.ToString(),
                            Path = row.Cells["ColPath"].Value.ToString(),
                            Type = row.Cells["ColType"].Value.ToString(),
                            SizeMB = double.Parse(row.Cells["ColSize"].Value.ToString()),
                            Estado = row.Cells["ColStatus"].Value.ToString()
                        });
                    }
                }

                var result = new ScanResult
                {
                    Items = items,
                    TotalSizeMB = totalSize / (1024.0 * 1024),
                    FreeSpaceMB = drive.AvailableFreeSpace / (1024.0 * 1024),
                    DiskTotalMB = drive.TotalSize / (1024.0 * 1024)
                };

                scanCache[rootPath] = result;

                if (!token.IsCancellationRequested)
                {
                    btnStop.Enabled = false;
                    btnScan.Enabled = true;
                    btnAtras.Enabled = true;
                    txtSize.ReadOnly = false;
                    txtPath.ReadOnly = false;
                    lblTotalSize.Text = $"Total ruta: {(totalSize / (1024.0 * 1024)).ToString("N2")} MB";
                    MessageBox.Show("Escaneo terminado.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                }
                else
                {
                    btnStop.Enabled = false;
                    btnScan.Enabled = true;
                    btnAtras.Enabled = false;
                    txtSize.ReadOnly = false;
                    txtPath.ReadOnly = false;
                    gridResults.Rows.Clear();
                    gridResults.Columns.Clear();
                    lblTotalSize.Text = "Escaneo cancelado";
                    MessageBox.Show("Escaneo cancelado.", "Mensaje", MessageBoxButtons.OK, MessageBoxIcon.Information);
                }
            }
            catch
            {
                lblTotalSize.Text = "Error durante el escaneo";
            }
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
                navigationHistory.Push(currentPath); // guardar la ruta actual
                string ruta = gridResults.SelectedRows[0].Cells["ColPath"].Value.ToString();
                txtPath.Text = ruta;
                EscanearRuta(true);
            }
        }

        private void escanearRutaToolStripMenuItem_DoubleClick(object sender, DataGridViewCellEventArgs e)
        {
            if (e.RowIndex < 0) return;

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
