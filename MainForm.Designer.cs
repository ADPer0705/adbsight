namespace android_debugger
{
    partial class MainForm
    {
        /// <summary>
        ///  Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        ///  Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        ///  Required method for Designer support - do not modify
        ///  the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            tabControlMain = new TabControl();
            tabSMS = new TabPage();
            panel2 = new Panel();
            dgvSMS = new DataGridView();
            Address = new DataGridViewTextBoxColumn();
            Body = new DataGridViewTextBoxColumn();
            SMSDate = new DataGridViewTextBoxColumn();
            SMSType = new DataGridViewTextBoxColumn();
            panel1 = new Panel();
            btnSaveSMS = new Button();
            btnLoadSMS = new Button();
            tabContacts = new TabPage();
            panel7 = new Panel();
            dgvContacts = new DataGridView();
            ContactName = new DataGridViewTextBoxColumn();
            PhoneNumber = new DataGridViewTextBoxColumn();
            Email = new DataGridViewTextBoxColumn();
            panel3 = new Panel();
            btnSaveContacts = new Button();
            btnLoadContacts = new Button();
            tabCalls = new TabPage();
            panel8 = new Panel();
            dgvCalls = new DataGridView();
            Number = new DataGridViewTextBoxColumn();
            CallType = new DataGridViewTextBoxColumn();
            CallDate = new DataGridViewTextBoxColumn();
            Duration = new DataGridViewTextBoxColumn();
            panel4 = new Panel();
            btnSaveCalls = new Button();
            btnLoadCalls = new Button();
            tabDevice = new TabPage();
            panel9 = new Panel();
            dgvDevice = new DataGridView();
            Key = new DataGridViewTextBoxColumn();
            Value = new DataGridViewTextBoxColumn();
            panel5 = new Panel();
            btnSaveDevice = new Button();
            btnLoadDevice = new Button();
            tabAccounts = new TabPage();
            panel10 = new Panel();
            dgvAccounts = new DataGridView();
            AccountType = new DataGridViewTextBoxColumn();
            AccountEmail = new DataGridViewTextBoxColumn();
            panel6 = new Panel();
            btnSaveAccount = new Button();
            btnLoadAccount = new Button();
            tabControlMain.SuspendLayout();
            tabSMS.SuspendLayout();
            panel2.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dgvSMS).BeginInit();
            panel1.SuspendLayout();
            tabContacts.SuspendLayout();
            panel7.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dgvContacts).BeginInit();
            panel3.SuspendLayout();
            tabCalls.SuspendLayout();
            panel8.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dgvCalls).BeginInit();
            panel4.SuspendLayout();
            tabDevice.SuspendLayout();
            panel9.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dgvDevice).BeginInit();
            panel5.SuspendLayout();
            tabAccounts.SuspendLayout();
            panel10.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)dgvAccounts).BeginInit();
            panel6.SuspendLayout();
            SuspendLayout();
            // 
            // tabControlMain
            // 
            tabControlMain.Controls.Add(tabSMS);
            tabControlMain.Controls.Add(tabContacts);
            tabControlMain.Controls.Add(tabCalls);
            tabControlMain.Controls.Add(tabDevice);
            tabControlMain.Controls.Add(tabAccounts);
            tabControlMain.Location = new Point(12, 12);
            tabControlMain.Name = "tabControlMain";
            tabControlMain.SelectedIndex = 0;
            tabControlMain.Size = new Size(1258, 829);
            tabControlMain.TabIndex = 0;
            // 
            // tabSMS
            // 
            tabSMS.Controls.Add(panel2);
            tabSMS.Controls.Add(panel1);
            tabSMS.Location = new Point(4, 29);
            tabSMS.Name = "tabSMS";
            tabSMS.Padding = new Padding(3);
            tabSMS.Size = new Size(1250, 796);
            tabSMS.TabIndex = 0;
            tabSMS.Text = "SMS";
            tabSMS.UseVisualStyleBackColor = true;
            // 
            // panel2
            // 
            panel2.Controls.Add(dgvSMS);
            panel2.Location = new Point(6, 49);
            panel2.Name = "panel2";
            panel2.Size = new Size(1238, 741);
            panel2.TabIndex = 1;
            // 
            // dgvSMS
            // 
            dgvSMS.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
            dgvSMS.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dgvSMS.Columns.AddRange(new DataGridViewColumn[] { Address, Body, SMSDate, SMSType });
            dgvSMS.Dock = DockStyle.Fill;
            dgvSMS.Location = new Point(0, 0);
            dgvSMS.Name = "dgvSMS";
            dgvSMS.RowHeadersWidth = 51;
            dgvSMS.Size = new Size(1238, 741);
            dgvSMS.TabIndex = 0;
            dgvSMS.CellContentClick += dataGridView1_CellContentClick;
            // 
            // Address
            // 
            Address.HeaderText = "From";
            Address.MinimumWidth = 6;
            Address.Name = "Address";
            // 
            // Body
            // 
            Body.HeaderText = "Message";
            Body.MinimumWidth = 6;
            Body.Name = "Body";
            // 
            // SMSDate
            // 
            SMSDate.HeaderText = "Date";
            SMSDate.MinimumWidth = 6;
            SMSDate.Name = "SMSDate";
            // 
            // SMSType
            // 
            SMSType.HeaderText = "Type";
            SMSType.MinimumWidth = 6;
            SMSType.Name = "SMSType";
            // 
            // panel1
            // 
            panel1.Controls.Add(btnSaveSMS);
            panel1.Controls.Add(btnLoadSMS);
            panel1.Location = new Point(6, 6);
            panel1.Name = "panel1";
            panel1.Size = new Size(1238, 37);
            panel1.TabIndex = 0;
            // 
            // btnSaveSMS
            // 
            btnSaveSMS.Location = new Point(159, 3);
            btnSaveSMS.Name = "btnSaveSMS";
            btnSaveSMS.Size = new Size(94, 29);
            btnSaveSMS.TabIndex = 1;
            btnSaveSMS.Text = "Save to DB";
            btnSaveSMS.UseVisualStyleBackColor = true;
            // 
            // btnLoadSMS
            // 
            btnLoadSMS.Location = new Point(3, 3);
            btnLoadSMS.Name = "btnLoadSMS";
            btnLoadSMS.Size = new Size(150, 29);
            btnLoadSMS.TabIndex = 0;
            btnLoadSMS.Text = "Load from Device";
            btnLoadSMS.UseVisualStyleBackColor = true;
            // 
            // tabContacts
            // 
            tabContacts.Controls.Add(panel7);
            tabContacts.Controls.Add(panel3);
            tabContacts.Location = new Point(4, 29);
            tabContacts.Name = "tabContacts";
            tabContacts.Padding = new Padding(3);
            tabContacts.Size = new Size(1250, 796);
            tabContacts.TabIndex = 1;
            tabContacts.Text = "Contacts";
            tabContacts.UseVisualStyleBackColor = true;
            // 
            // panel7
            // 
            panel7.Controls.Add(dgvContacts);
            panel7.Location = new Point(6, 49);
            panel7.Name = "panel7";
            panel7.Size = new Size(1238, 741);
            panel7.TabIndex = 3;
            // 
            // dgvContacts
            // 
            dgvContacts.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
            dgvContacts.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dgvContacts.Columns.AddRange(new DataGridViewColumn[] { ContactName, PhoneNumber, Email });
            dgvContacts.Dock = DockStyle.Fill;
            dgvContacts.Location = new Point(0, 0);
            dgvContacts.Name = "dgvContacts";
            dgvContacts.RowHeadersWidth = 51;
            dgvContacts.Size = new Size(1238, 741);
            dgvContacts.TabIndex = 0;
            // 
            // ContactName
            // 
            ContactName.HeaderText = "Name";
            ContactName.MinimumWidth = 6;
            ContactName.Name = "ContactName";
            // 
            // PhoneNumber
            // 
            PhoneNumber.HeaderText = "Phone Number";
            PhoneNumber.MinimumWidth = 6;
            PhoneNumber.Name = "PhoneNumber";
            // 
            // Email
            // 
            Email.HeaderText = "Email";
            Email.MinimumWidth = 6;
            Email.Name = "Email";
            // 
            // panel3
            // 
            panel3.Controls.Add(btnSaveContacts);
            panel3.Controls.Add(btnLoadContacts);
            panel3.Location = new Point(6, 6);
            panel3.Name = "panel3";
            panel3.Size = new Size(1238, 37);
            panel3.TabIndex = 1;
            // 
            // btnSaveContacts
            // 
            btnSaveContacts.Location = new Point(159, 3);
            btnSaveContacts.Name = "btnSaveContacts";
            btnSaveContacts.Size = new Size(94, 29);
            btnSaveContacts.TabIndex = 1;
            btnSaveContacts.Text = "Save to DB";
            btnSaveContacts.UseVisualStyleBackColor = true;
            // 
            // btnLoadContacts
            // 
            btnLoadContacts.Location = new Point(3, 3);
            btnLoadContacts.Name = "btnLoadContacts";
            btnLoadContacts.Size = new Size(150, 29);
            btnLoadContacts.TabIndex = 0;
            btnLoadContacts.Text = "Load from Device";
            btnLoadContacts.UseVisualStyleBackColor = true;
            // 
            // tabCalls
            // 
            tabCalls.Controls.Add(panel8);
            tabCalls.Controls.Add(panel4);
            tabCalls.Location = new Point(4, 29);
            tabCalls.Name = "tabCalls";
            tabCalls.Padding = new Padding(3);
            tabCalls.Size = new Size(1250, 796);
            tabCalls.TabIndex = 2;
            tabCalls.Text = "Call Logs";
            tabCalls.UseVisualStyleBackColor = true;
            // 
            // panel8
            // 
            panel8.Controls.Add(dgvCalls);
            panel8.Location = new Point(6, 49);
            panel8.Name = "panel8";
            panel8.Size = new Size(1238, 741);
            panel8.TabIndex = 2;
            // 
            // dgvCalls
            // 
            dgvCalls.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
            dgvCalls.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dgvCalls.Columns.AddRange(new DataGridViewColumn[] { Number, CallType, CallDate, Duration });
            dgvCalls.Dock = DockStyle.Fill;
            dgvCalls.Location = new Point(0, 0);
            dgvCalls.Name = "dgvCalls";
            dgvCalls.RowHeadersWidth = 51;
            dgvCalls.Size = new Size(1238, 741);
            dgvCalls.TabIndex = 0;
            // 
            // Number
            // 
            Number.HeaderText = "Phone Number";
            Number.MinimumWidth = 6;
            Number.Name = "Number";
            // 
            // CallType
            // 
            CallType.HeaderText = "Type";
            CallType.MinimumWidth = 6;
            CallType.Name = "CallType";
            // 
            // CallDate
            // 
            CallDate.HeaderText = "Date";
            CallDate.MinimumWidth = 6;
            CallDate.Name = "CallDate";
            // 
            // Duration
            // 
            Duration.HeaderText = "Duration (Sec)";
            Duration.MinimumWidth = 6;
            Duration.Name = "Duration";
            // 
            // panel4
            // 
            panel4.Controls.Add(btnSaveCalls);
            panel4.Controls.Add(btnLoadCalls);
            panel4.Location = new Point(6, 6);
            panel4.Name = "panel4";
            panel4.Size = new Size(1238, 37);
            panel4.TabIndex = 1;
            // 
            // btnSaveCalls
            // 
            btnSaveCalls.Location = new Point(159, 3);
            btnSaveCalls.Name = "btnSaveCalls";
            btnSaveCalls.Size = new Size(94, 29);
            btnSaveCalls.TabIndex = 1;
            btnSaveCalls.Text = "Save to DB";
            btnSaveCalls.UseVisualStyleBackColor = true;
            // 
            // btnLoadCalls
            // 
            btnLoadCalls.Location = new Point(3, 3);
            btnLoadCalls.Name = "btnLoadCalls";
            btnLoadCalls.Size = new Size(150, 29);
            btnLoadCalls.TabIndex = 0;
            btnLoadCalls.Text = "Load from Device";
            btnLoadCalls.UseVisualStyleBackColor = true;
            // 
            // tabDevice
            // 
            tabDevice.Controls.Add(panel9);
            tabDevice.Controls.Add(panel5);
            tabDevice.Location = new Point(4, 29);
            tabDevice.Name = "tabDevice";
            tabDevice.Padding = new Padding(3);
            tabDevice.Size = new Size(1250, 796);
            tabDevice.TabIndex = 3;
            tabDevice.Text = "Device Info";
            tabDevice.UseVisualStyleBackColor = true;
            // 
            // panel9
            // 
            panel9.Controls.Add(dgvDevice);
            panel9.Location = new Point(6, 49);
            panel9.Name = "panel9";
            panel9.Size = new Size(1238, 741);
            panel9.TabIndex = 2;
            // 
            // dgvDevice
            // 
            dgvDevice.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
            dgvDevice.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dgvDevice.Columns.AddRange(new DataGridViewColumn[] { Key, Value });
            dgvDevice.Dock = DockStyle.Fill;
            dgvDevice.Location = new Point(0, 0);
            dgvDevice.Name = "dgvDevice";
            dgvDevice.RowHeadersWidth = 51;
            dgvDevice.Size = new Size(1238, 741);
            dgvDevice.TabIndex = 0;
            // 
            // Key
            // 
            Key.HeaderText = "Property";
            Key.MinimumWidth = 6;
            Key.Name = "Key";
            // 
            // Value
            // 
            Value.HeaderText = "Value";
            Value.MinimumWidth = 6;
            Value.Name = "Value";
            // 
            // panel5
            // 
            panel5.Controls.Add(btnSaveDevice);
            panel5.Controls.Add(btnLoadDevice);
            panel5.Location = new Point(6, 6);
            panel5.Name = "panel5";
            panel5.Size = new Size(1238, 37);
            panel5.TabIndex = 1;
            // 
            // btnSaveDevice
            // 
            btnSaveDevice.Location = new Point(159, 3);
            btnSaveDevice.Name = "btnSaveDevice";
            btnSaveDevice.Size = new Size(94, 29);
            btnSaveDevice.TabIndex = 1;
            btnSaveDevice.Text = "Save to DB";
            btnSaveDevice.UseVisualStyleBackColor = true;
            // 
            // btnLoadDevice
            // 
            btnLoadDevice.Location = new Point(3, 3);
            btnLoadDevice.Name = "btnLoadDevice";
            btnLoadDevice.Size = new Size(150, 29);
            btnLoadDevice.TabIndex = 0;
            btnLoadDevice.Text = "Load from Device";
            btnLoadDevice.UseVisualStyleBackColor = true;
            // 
            // tabAccounts
            // 
            tabAccounts.Controls.Add(panel10);
            tabAccounts.Controls.Add(panel6);
            tabAccounts.Location = new Point(4, 29);
            tabAccounts.Name = "tabAccounts";
            tabAccounts.Padding = new Padding(3);
            tabAccounts.Size = new Size(1250, 796);
            tabAccounts.TabIndex = 4;
            tabAccounts.Text = "Accounts";
            tabAccounts.UseVisualStyleBackColor = true;
            // 
            // panel10
            // 
            panel10.Controls.Add(dgvAccounts);
            panel10.Location = new Point(6, 49);
            panel10.Name = "panel10";
            panel10.Size = new Size(1238, 741);
            panel10.TabIndex = 2;
            // 
            // dgvAccounts
            // 
            dgvAccounts.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
            dgvAccounts.ColumnHeadersHeightSizeMode = DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            dgvAccounts.Columns.AddRange(new DataGridViewColumn[] { AccountType, AccountEmail });
            dgvAccounts.Dock = DockStyle.Fill;
            dgvAccounts.Location = new Point(0, 0);
            dgvAccounts.Name = "dgvAccounts";
            dgvAccounts.RowHeadersWidth = 51;
            dgvAccounts.Size = new Size(1238, 741);
            dgvAccounts.TabIndex = 0;
            // 
            // AccountType
            // 
            AccountType.HeaderText = "Account Type";
            AccountType.MinimumWidth = 6;
            AccountType.Name = "AccountType";
            // 
            // AccountEmail
            // 
            AccountEmail.HeaderText = "Email";
            AccountEmail.MinimumWidth = 6;
            AccountEmail.Name = "AccountEmail";
            // 
            // panel6
            // 
            panel6.Controls.Add(btnSaveAccount);
            panel6.Controls.Add(btnLoadAccount);
            panel6.Location = new Point(6, 6);
            panel6.Name = "panel6";
            panel6.Size = new Size(1238, 37);
            panel6.TabIndex = 1;
            // 
            // btnSaveAccount
            // 
            btnSaveAccount.Location = new Point(159, 3);
            btnSaveAccount.Name = "btnSaveAccount";
            btnSaveAccount.Size = new Size(94, 29);
            btnSaveAccount.TabIndex = 1;
            btnSaveAccount.Text = "Save to DB";
            btnSaveAccount.UseVisualStyleBackColor = true;
            // 
            // btnLoadAccount
            // 
            btnLoadAccount.Location = new Point(3, 3);
            btnLoadAccount.Name = "btnLoadAccount";
            btnLoadAccount.Size = new Size(150, 29);
            btnLoadAccount.TabIndex = 0;
            btnLoadAccount.Text = "Load from Device";
            btnLoadAccount.UseVisualStyleBackColor = true;
            //            // MainForm
            // 
            AutoScaleDimensions = new SizeF(8F, 20F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(1282, 853);
            Controls.Add(tabControlMain);
            Name = "MainForm";
            Text = "Android Debugger";
            tabControlMain.ResumeLayout(false);
            tabSMS.ResumeLayout(false);
            panel2.ResumeLayout(false);
            ((System.ComponentModel.ISupportInitialize)dgvSMS).EndInit();
            panel1.ResumeLayout(false);
            tabContacts.ResumeLayout(false);
            panel7.ResumeLayout(false);
            ((System.ComponentModel.ISupportInitialize)dgvContacts).EndInit();
            panel3.ResumeLayout(false);
            tabCalls.ResumeLayout(false);
            panel8.ResumeLayout(false);
            ((System.ComponentModel.ISupportInitialize)dgvCalls).EndInit();
            panel4.ResumeLayout(false);
            tabDevice.ResumeLayout(false);
            panel9.ResumeLayout(false);
            ((System.ComponentModel.ISupportInitialize)dgvDevice).EndInit();
            panel5.ResumeLayout(false);
            tabAccounts.ResumeLayout(false);
            panel10.ResumeLayout(false);
            ((System.ComponentModel.ISupportInitialize)dgvAccounts).EndInit();
            panel6.ResumeLayout(false);
            ResumeLayout(false);
        }

        #endregion

        private TabControl tabControlMain;
        private TabPage tabSMS;
        private TabPage tabContacts;
        private TabPage tabCalls;
        private TabPage tabDevice;
        private TabPage tabAccounts;
        private Panel panel1;
        private Button btnSaveSMS;
        private Button btnLoadSMS;
        private Panel panel2;
        private DataGridView dgvSMS;
        private Panel panel7;
        private Panel panel3;
        private Button btnSaveContacts;
        private Button btnLoadContacts;
        private Panel panel4;
        private Button btnSaveCalls;
        private Button btnLoadCalls;
        private Panel panel5;
        private Button btnSaveDevice;
        private Button btnLoadDevice;
        private Panel panel6;
        private Button btnSaveAccount;
        private Button btnLoadAccount;
        private DataGridView dgvContacts;
        private Panel panel8;
        private Panel panel9;
        private Panel panel10;
        private DataGridView dgvCalls;
        private DataGridView dgvDevice;
        private DataGridView dgvAccounts;        private DataGridViewTextBoxColumn ContactName; // Changed from Name to avoid conflict with Control.Name
        private DataGridViewTextBoxColumn PhoneNumber;
        private DataGridViewTextBoxColumn Email;
        private DataGridViewTextBoxColumn Number;
        private DataGridViewTextBoxColumn CallType;
        private DataGridViewTextBoxColumn CallDate;
        private DataGridViewTextBoxColumn Duration;
        private DataGridViewTextBoxColumn Address;
        private DataGridViewTextBoxColumn Body;
        private DataGridViewTextBoxColumn SMSDate;
        private DataGridViewTextBoxColumn SMSType;
        private DataGridViewTextBoxColumn Key;
        private DataGridViewTextBoxColumn Value;
        private DataGridViewTextBoxColumn AccountType;
        private DataGridViewTextBoxColumn AccountEmail;
    }
}
