namespace SpaceScanner
{
    partial class Form1
    {
        /// <summary>
        /// Variable del diseñador necesaria.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Limpiar los recursos que se estén usando.
        /// </summary>
        /// <param name="disposing">true si los recursos administrados se deben desechar; false en caso contrario.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Código generado por el Diseñador de Windows Forms

        /// <summary>
        /// Método necesario para admitir el Diseñador. No se puede modificar
        /// el contenido de este método con el editor de código.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(Form1));
            this.txtPath = new System.Windows.Forms.TextBox();
            this.label1 = new System.Windows.Forms.Label();
            this.btnScan = new System.Windows.Forms.Button();
            this.gridResults = new System.Windows.Forms.DataGridView();
            this.lblTotalSize = new System.Windows.Forms.Label();
            this.lblFreeSize = new System.Windows.Forms.Label();
            this.lblGlobalSize = new System.Windows.Forms.Label();
            this.btnStop = new System.Windows.Forms.Button();
            this.contextMenuGrid = new System.Windows.Forms.ContextMenuStrip(this.components);
            this.abrirCarpetaToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.escanearRutaToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.label2 = new System.Windows.Forms.Label();
            this.txtSize = new System.Windows.Forms.TextBox();
            this.btnAtras = new System.Windows.Forms.Button();
            this.txtCacheIndicator = new System.Windows.Forms.Label();
            this.menuStrip1 = new System.Windows.Forms.MenuStrip();
            this.detallesToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.vistaDetalladaToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.vistaGráficaToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.graphPanel = new System.Windows.Forms.Panel();
            ((System.ComponentModel.ISupportInitialize)(this.gridResults)).BeginInit();
            this.contextMenuGrid.SuspendLayout();
            this.menuStrip1.SuspendLayout();
            this.SuspendLayout();
            // 
            // txtPath
            // 
            this.txtPath.Location = new System.Drawing.Point(123, 54);
            this.txtPath.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.txtPath.Name = "txtPath";
            this.txtPath.Size = new System.Drawing.Size(966, 29);
            this.txtPath.TabIndex = 0;
            this.txtPath.KeyDown += new System.Windows.Forms.KeyEventHandler(this.txtPath_KeyDown);
            // 
            // label1
            // 
            this.label1.AutoSize = true;
            this.label1.Location = new System.Drawing.Point(51, 59);
            this.label1.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.label1.Name = "label1";
            this.label1.Size = new System.Drawing.Size(58, 25);
            this.label1.TabIndex = 1;
            this.label1.Text = "Ruta:";
            // 
            // btnScan
            // 
            this.btnScan.BackColor = System.Drawing.SystemColors.Control;
            this.btnScan.Location = new System.Drawing.Point(1104, 52);
            this.btnScan.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.btnScan.Name = "btnScan";
            this.btnScan.Size = new System.Drawing.Size(145, 42);
            this.btnScan.TabIndex = 2;
            this.btnScan.Text = "Escanear";
            this.btnScan.UseVisualStyleBackColor = false;
            this.btnScan.Click += new System.EventHandler(this.btnScan_Click);
            // 
            // gridResults
            // 
            this.gridResults.AutoSizeColumnsMode = System.Windows.Forms.DataGridViewAutoSizeColumnsMode.Fill;
            this.gridResults.ColumnHeadersHeightSizeMode = System.Windows.Forms.DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            this.gridResults.Location = new System.Drawing.Point(57, 159);
            this.gridResults.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.gridResults.Name = "gridResults";
            this.gridResults.ReadOnly = true;
            this.gridResults.RowHeadersWidth = 72;
            this.gridResults.Size = new System.Drawing.Size(1287, 478);
            this.gridResults.TabIndex = 3;
            this.gridResults.CellContentDoubleClick += new System.Windows.Forms.DataGridViewCellEventHandler(this.escanearRutaToolStripMenuItem_DoubleClick);
            // 
            // lblTotalSize
            // 
            this.lblTotalSize.AutoSize = true;
            this.lblTotalSize.Location = new System.Drawing.Point(57, 648);
            this.lblTotalSize.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.lblTotalSize.Name = "lblTotalSize";
            this.lblTotalSize.Size = new System.Drawing.Size(133, 25);
            this.lblTotalSize.TabIndex = 6;
            this.lblTotalSize.Text = "total de la ruta";
            // 
            // lblFreeSize
            // 
            this.lblFreeSize.AutoSize = true;
            this.lblFreeSize.Location = new System.Drawing.Point(1109, 648);
            this.lblFreeSize.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.lblFreeSize.Name = "lblFreeSize";
            this.lblFreeSize.Size = new System.Drawing.Size(121, 25);
            this.lblFreeSize.TabIndex = 7;
            this.lblFreeSize.Text = "espacio libre";
            // 
            // lblGlobalSize
            // 
            this.lblGlobalSize.AutoSize = true;
            this.lblGlobalSize.Location = new System.Drawing.Point(1109, 685);
            this.lblGlobalSize.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.lblGlobalSize.Name = "lblGlobalSize";
            this.lblGlobalSize.Size = new System.Drawing.Size(130, 25);
            this.lblGlobalSize.TabIndex = 8;
            this.lblGlobalSize.Text = "total del disco";
            // 
            // btnStop
            // 
            this.btnStop.BackColor = System.Drawing.SystemColors.Control;
            this.btnStop.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(255)))), ((int)(((byte)(192)))), ((int)(((byte)(192)))));
            this.btnStop.Location = new System.Drawing.Point(1250, 52);
            this.btnStop.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.btnStop.Name = "btnStop";
            this.btnStop.Size = new System.Drawing.Size(92, 42);
            this.btnStop.TabIndex = 9;
            this.btnStop.Text = "Parar";
            this.btnStop.UseVisualStyleBackColor = false;
            this.btnStop.Click += new System.EventHandler(this.btnStop_Click);
            // 
            // contextMenuGrid
            // 
            this.contextMenuGrid.ImageScalingSize = new System.Drawing.Size(28, 28);
            this.contextMenuGrid.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.abrirCarpetaToolStripMenuItem,
            this.escanearRutaToolStripMenuItem});
            this.contextMenuGrid.Name = "contextMenuGrid";
            this.contextMenuGrid.Size = new System.Drawing.Size(212, 76);
            // 
            // abrirCarpetaToolStripMenuItem
            // 
            this.abrirCarpetaToolStripMenuItem.Name = "abrirCarpetaToolStripMenuItem";
            this.abrirCarpetaToolStripMenuItem.Size = new System.Drawing.Size(211, 36);
            this.abrirCarpetaToolStripMenuItem.Text = "Abrir recurso";
            this.abrirCarpetaToolStripMenuItem.Click += new System.EventHandler(this.abrirRecursoToolStripMenuItem_Click);
            // 
            // escanearRutaToolStripMenuItem
            // 
            this.escanearRutaToolStripMenuItem.Name = "escanearRutaToolStripMenuItem";
            this.escanearRutaToolStripMenuItem.Size = new System.Drawing.Size(211, 36);
            this.escanearRutaToolStripMenuItem.Text = "Escanear ruta";
            this.escanearRutaToolStripMenuItem.Click += new System.EventHandler(this.escanearRutaToolStripMenuItem_Click);
            // 
            // label2
            // 
            this.label2.AutoSize = true;
            this.label2.Location = new System.Drawing.Point(51, 111);
            this.label2.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.label2.Name = "label2";
            this.label2.Size = new System.Drawing.Size(69, 25);
            this.label2.TabIndex = 10;
            this.label2.Text = "Límite:";
            // 
            // txtSize
            // 
            this.txtSize.Location = new System.Drawing.Point(123, 111);
            this.txtSize.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.txtSize.Name = "txtSize";
            this.txtSize.Size = new System.Drawing.Size(169, 29);
            this.txtSize.TabIndex = 11;
            // 
            // btnAtras
            // 
            this.btnAtras.BackColor = System.Drawing.SystemColors.Control;
            this.btnAtras.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(255)))), ((int)(((byte)(192)))), ((int)(((byte)(192)))));
            this.btnAtras.Location = new System.Drawing.Point(1250, 102);
            this.btnAtras.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.btnAtras.Name = "btnAtras";
            this.btnAtras.Size = new System.Drawing.Size(92, 42);
            this.btnAtras.TabIndex = 12;
            this.btnAtras.Text = "Atrás";
            this.btnAtras.UseVisualStyleBackColor = false;
            this.btnAtras.Click += new System.EventHandler(this.btnBack_Click);
            // 
            // txtCacheIndicator
            // 
            this.txtCacheIndicator.AutoSize = true;
            this.txtCacheIndicator.Font = new System.Drawing.Font("Microsoft Sans Serif", 6F, ((System.Drawing.FontStyle)((System.Drawing.FontStyle.Bold | System.Drawing.FontStyle.Italic))), System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.txtCacheIndicator.ForeColor = System.Drawing.Color.Red;
            this.txtCacheIndicator.Location = new System.Drawing.Point(1014, 124);
            this.txtCacheIndicator.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.txtCacheIndicator.Name = "txtCacheIndicator";
            this.txtCacheIndicator.Size = new System.Drawing.Size(219, 17);
            this.txtCacheIndicator.TabIndex = 13;
            this.txtCacheIndicator.Text = "Datos cargados desde caché";
            // 
            // menuStrip1
            // 
            this.menuStrip1.GripMargin = new System.Windows.Forms.Padding(2, 2, 0, 2);
            this.menuStrip1.ImageScalingSize = new System.Drawing.Size(28, 28);
            this.menuStrip1.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.detallesToolStripMenuItem});
            this.menuStrip1.Location = new System.Drawing.Point(0, 0);
            this.menuStrip1.Name = "menuStrip1";
            this.menuStrip1.Padding = new System.Windows.Forms.Padding(11, 4, 0, 4);
            this.menuStrip1.Size = new System.Drawing.Size(1390, 42);
            this.menuStrip1.TabIndex = 14;
            this.menuStrip1.Text = "menuStrip1";
            // 
            // detallesToolStripMenuItem
            // 
            this.detallesToolStripMenuItem.DropDownItems.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.vistaDetalladaToolStripMenuItem,
            this.vistaGráficaToolStripMenuItem});
            this.detallesToolStripMenuItem.Name = "detallesToolStripMenuItem";
            this.detallesToolStripMenuItem.Size = new System.Drawing.Size(61, 34);
            this.detallesToolStripMenuItem.Text = "Ver";
            // 
            // vistaDetalladaToolStripMenuItem
            // 
            this.vistaDetalladaToolStripMenuItem.Name = "vistaDetalladaToolStripMenuItem";
            this.vistaDetalladaToolStripMenuItem.Size = new System.Drawing.Size(267, 40);
            this.vistaDetalladaToolStripMenuItem.Text = "Vista detallada";
            this.vistaDetalladaToolStripMenuItem.Click += new System.EventHandler(this.vistaDetalladaToolStripMenuItem_Click);
            // 
            // vistaGráficaToolStripMenuItem
            // 
            this.vistaGráficaToolStripMenuItem.Name = "vistaGráficaToolStripMenuItem";
            this.vistaGráficaToolStripMenuItem.Size = new System.Drawing.Size(267, 40);
            this.vistaGráficaToolStripMenuItem.Text = "Vista gráfica";
            this.vistaGráficaToolStripMenuItem.Click += new System.EventHandler(this.vistaGraficaToolStripMenuItem_Click);
            // 
            // graphPanel
            // 
            this.graphPanel.Location = new System.Drawing.Point(1325, 50);
            this.graphPanel.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.graphPanel.Name = "graphPanel";
            this.graphPanel.Size = new System.Drawing.Size(65, 678);
            this.graphPanel.TabIndex = 15;
            // 
            // Form1
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(11F, 24F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(1390, 724);
            this.Controls.Add(this.graphPanel);
            this.Controls.Add(this.menuStrip1);
            this.Controls.Add(this.txtCacheIndicator);
            this.Controls.Add(this.btnAtras);
            this.Controls.Add(this.txtSize);
            this.Controls.Add(this.label2);
            this.Controls.Add(this.btnStop);
            this.Controls.Add(this.lblGlobalSize);
            this.Controls.Add(this.lblFreeSize);
            this.Controls.Add(this.lblTotalSize);
            this.Controls.Add(this.gridResults);
            this.Controls.Add(this.btnScan);
            this.Controls.Add(this.label1);
            this.Controls.Add(this.txtPath);
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MainMenuStrip = this.menuStrip1;
            this.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.Name = "Form1";
            this.Text = "SpaceScanner";
            ((System.ComponentModel.ISupportInitialize)(this.gridResults)).EndInit();
            this.contextMenuGrid.ResumeLayout(false);
            this.menuStrip1.ResumeLayout(false);
            this.menuStrip1.PerformLayout();
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.TextBox txtPath;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.Button btnScan;
        private System.Windows.Forms.DataGridView gridResults;
        private System.Windows.Forms.Label lblTotalSize;
        private System.Windows.Forms.Label lblFreeSize;
        private System.Windows.Forms.Label lblGlobalSize;
        private System.Windows.Forms.Button btnStop;
        private System.Windows.Forms.ContextMenuStrip contextMenuGrid;
        private System.Windows.Forms.ToolStripMenuItem abrirCarpetaToolStripMenuItem;
        private System.Windows.Forms.ToolStripMenuItem escanearRutaToolStripMenuItem;
        private System.Windows.Forms.Label label2;
        private System.Windows.Forms.TextBox txtSize;
        private System.Windows.Forms.Button btnAtras;
        private System.Windows.Forms.Label txtCacheIndicator;
        private System.Windows.Forms.MenuStrip menuStrip1;
        private System.Windows.Forms.ToolStripMenuItem detallesToolStripMenuItem;
        private System.Windows.Forms.ToolStripMenuItem vistaDetalladaToolStripMenuItem;
        private System.Windows.Forms.ToolStripMenuItem vistaGráficaToolStripMenuItem;
        private System.Windows.Forms.Panel graphPanel;
    }
}

