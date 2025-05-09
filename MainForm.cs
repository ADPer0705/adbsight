using android_debugger.Data;
using android_debugger.Models;
using android_debugger.Services;
using android_debugger.Utils;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using System.ComponentModel;
using System.Diagnostics;

namespace android_debugger
{
    public partial class MainForm : Form
    {
        private readonly AdbTool _adbTool;
        private readonly DataService _dataService;
        private readonly IServiceProvider _serviceProvider;

        // Data collections
        private List<Contact> _contacts = new();
        private List<SMS> _smsMessages = new();
        private List<CallLog> _callLogs = new();
        private List<DeviceInfo> _deviceInfo = new();
        private List<Account> _accounts = new(); public MainForm()
        {
            InitializeComponent();

            // Setup dependency injection
            var services = new ServiceCollection();
            ConfigureServices(services);
            _serviceProvider = services.BuildServiceProvider();

            // Get instances of services
            _adbTool = new AdbTool(AppSettings.AdbPath);
            _dataService = _serviceProvider.GetRequiredService<DataService>();

            // Create context menus for data grids
            SetupContextMenus();            // Setup event handlers
            this.Load += MainForm_Load;
            btnLoadSMS.Click += BtnLoadSMS_Click;
            btnSaveSMS.Click += BtnSaveSMS_Click;
            btnLoadContacts.Click += BtnLoadContacts_Click;
            btnSaveContacts.Click += BtnSaveContacts_Click; btnLoadCalls.Click += BtnLoadCalls_Click;
            btnSaveCalls.Click += BtnSaveCalls_Click;
            btnLoadDevice.Click += BtnLoadDevice_Click; // Device Info Load            btnSaveDevice.Click += BtnSaveDevice_Click; // Device Info Save
            btnLoadAccount.Click += BtnLoadAccounts_Click; // Accounts Load
            btnSaveAccount.Click += BtnSaveAccounts_Click; // Accounts Save
        }
        private void SetupContextMenus()
        {
            // Create context menu for SMS data grid - simplified for now
            var smsContextMenu = new ContextMenuStrip();
            dgvSMS.ContextMenuStrip = smsContextMenu;

            // Remove other context menus for now since we're focusing only on SMS
        }

        private void ConfigureServices(ServiceCollection services)
        {
            // Add database context
            services.AddDbContext<AndroidDbContext>(options =>
                options.UseSqlServer(GetConnectionString()),
                ServiceLifetime.Transient);

            // Add data service
            services.AddTransient<DataService>();
        }
        private string GetConnectionString()
        {
            // Use the AppSettings utility to get the connection string
            return AppSettings.GetConnectionString();
        }

        private void MainForm_Load(object? sender, EventArgs e)
        {
            try
            {
                // Initialize database
                using var scope = _serviceProvider.CreateScope();
                var dbContext = scope.ServiceProvider.GetRequiredService<AndroidDbContext>();
                dbContext.Database.EnsureCreated();

                // Load data from database if available
                LoadDataFromDatabase();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error initializing the application: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }
        private async void LoadDataFromDatabase()
        {
            try
            {
                // Load all data types from database
                _smsMessages = await _dataService.GetSMSAsync();
                _contacts = await _dataService.GetContactsAsync();
                _callLogs = await _dataService.GetCallLogsAsync();
                _deviceInfo = await _dataService.GetDeviceInfoAsync();
                _accounts = await _dataService.GetAccountsAsync();

                // Update all UI components
                UpdateSMSDisplay();
                UpdateContactsDisplay();
                UpdateCallLogsDisplay();
                UpdateDeviceInfoDisplay();
                UpdateAccountsDisplay();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error loading data from database: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        #region SMS Tab

        private async void BtnLoadSMS_Click(object? sender, EventArgs e)
        {
            try
            {
                if (!_adbTool.IsDeviceConnected())
                {
                    MessageBox.Show("No Android device connected. Please connect a device and try again.",
                        "Device Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnLoadSMS.Enabled = false;
                btnLoadSMS.Text = "Loading...";

                // Load SMS messages from device
                _smsMessages = await Task.Run(() => _adbTool.GetSMS());

                // Update the UI
                UpdateSMSDisplay();

                btnLoadSMS.Enabled = true;
                btnLoadSMS.Text = "Load from Device";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully loaded {_smsMessages.Count} SMS messages from the device.",
                    "SMS Loaded", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnLoadSMS.Enabled = true;
                btnLoadSMS.Text = "Load from Device";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error loading SMS messages: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async void BtnSaveSMS_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_smsMessages.Count == 0)
                {
                    MessageBox.Show("No SMS messages to save. Please load SMS messages first.",
                        "No Data", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnSaveSMS.Enabled = false;
                btnSaveSMS.Text = "Saving...";

                // Save SMS messages to database
                await _dataService.SaveSMSAsync(_smsMessages);

                btnSaveSMS.Enabled = true;
                btnSaveSMS.Text = "Save to DB";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully saved {_smsMessages.Count} SMS messages to the database.",
                    "SMS Saved", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnSaveSMS.Enabled = true;
                btnSaveSMS.Text = "Save to DB";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error saving SMS messages: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }
        private void UpdateSMSDisplay()
        {
            // Clear existing data
            dgvSMS.Rows.Clear();

            // Add SMS messages to the data grid
            foreach (var sms in _smsMessages)
            {
                dgvSMS.Rows.Add(
                    sms.Address,
                    sms.Body,
                    sms.Date.ToString("yyyy-MM-dd HH:mm:ss"),
                    sms.Type
                );
            }
        }

        private void dataGridView1_CellContentClick(object sender, DataGridViewCellEventArgs e)
        {
            // This is a placeholder for future functionality
            // We might implement message details view or other actions here
        }

        #endregion

        #region Contacts Tab

        private async void BtnLoadContacts_Click(object? sender, EventArgs e)
        {
            try
            {
                if (!_adbTool.IsDeviceConnected())
                {
                    MessageBox.Show("No Android device connected. Please connect a device and try again.",
                        "Device Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnLoadContacts.Enabled = false;
                btnLoadContacts.Text = "Loading...";

                // Load contacts from device
                _contacts = await Task.Run(() => _adbTool.GetContacts());

                // Update the UI
                UpdateContactsDisplay();

                btnLoadContacts.Enabled = true;
                btnLoadContacts.Text = "Load from Device";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully loaded {_contacts.Count} contacts from the device.",
                    "Contacts Loaded", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnLoadContacts.Enabled = true;
                btnLoadContacts.Text = "Load from Device";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error loading contacts: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async void BtnSaveContacts_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_contacts.Count == 0)
                {
                    MessageBox.Show("No contacts to save. Please load contacts first.",
                        "No Data", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnSaveContacts.Enabled = false;
                btnSaveContacts.Text = "Saving...";

                // Save contacts to database
                await _dataService.SaveContactsAsync(_contacts);

                btnSaveContacts.Enabled = true;
                btnSaveContacts.Text = "Save to DB";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully saved {_contacts.Count} contacts to the database.",
                    "Contacts Saved", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnSaveContacts.Enabled = true;
                btnSaveContacts.Text = "Save to DB";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error saving contacts: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void UpdateContactsDisplay()
        {
            // Clear existing data
            dgvContacts.Rows.Clear();

            // Add contacts to the data grid
            foreach (var contact in _contacts)
            {
                dgvContacts.Rows.Add(
                    contact.Name,
                    contact.PhoneNumber,
                    contact.Email
                );
            }
        }

        #endregion

        #region Call Logs Tab

        private async void BtnLoadCalls_Click(object? sender, EventArgs e)
        {
            try
            {
                if (!_adbTool.IsDeviceConnected())
                {
                    MessageBox.Show("No Android device connected. Please connect a device and try again.",
                        "Device Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnLoadCalls.Enabled = false;
                btnLoadCalls.Text = "Loading...";

                // Load call logs from device
                _callLogs = await Task.Run(() => _adbTool.GetCallLogs());

                // Update the UI
                UpdateCallLogsDisplay();

                btnLoadCalls.Enabled = true;
                btnLoadCalls.Text = "Load from Device";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully loaded {_callLogs.Count} call logs from the device.",
                    "Call Logs Loaded", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnLoadCalls.Enabled = true;
                btnLoadCalls.Text = "Load from Device";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error loading call logs: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async void BtnSaveCalls_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_callLogs.Count == 0)
                {
                    MessageBox.Show("No call logs to save. Please load call logs first.",
                        "No Data", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }

                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnSaveCalls.Enabled = false;
                btnSaveCalls.Text = "Saving...";

                // Save call logs to database
                await _dataService.SaveCallLogsAsync(_callLogs);

                btnSaveCalls.Enabled = true;
                btnSaveCalls.Text = "Save to DB";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully saved {_callLogs.Count} call logs to the database.",
                    "Call Logs Saved", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnSaveCalls.Enabled = true;
                btnSaveCalls.Text = "Save to DB";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error saving call logs: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void UpdateCallLogsDisplay()
        {
            // Clear existing data
            dgvCalls.Rows.Clear();

            // Add call logs to the data grid
            foreach (var call in _callLogs)
            {
                dgvCalls.Rows.Add(
                    call.Number,
                    call.CallType,
                    call.Date.ToString("yyyy-MM-dd HH:mm:ss"),
                    call.Duration.ToString()
                );
            }
        }

        #endregion

        #region Device Info Tab

        private async void BtnLoadDevice_Click(object? sender, EventArgs e)
        {
            try
            {
                if (!_adbTool.IsDeviceConnected())
                {
                    MessageBox.Show("No Android device connected. Please connect a device and try again.",
                        "Device Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                    return;
                }                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnLoadDevice.Enabled = false;
                btnLoadDevice.Text = "Loading...";

                // Load device information from device
                _deviceInfo = await Task.Run(() => _adbTool.GetDeviceInfo());

                // Update the UI
                UpdateDeviceInfoDisplay();

                btnLoadDevice.Enabled = true;
                btnLoadDevice.Text = "Load from Device";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully loaded device information from the device.",
                    "Device Info Loaded", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnLoadDevice.Enabled = true;
                btnLoadDevice.Text = "Load from Device";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error loading device information: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async void BtnSaveDevice_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_deviceInfo.Count == 0)
                {
                    MessageBox.Show("No device information to save. Please load device information first.",
                        "No Data", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnSaveDevice.Enabled = false;
                btnSaveDevice.Text = "Saving...";

                // Save device information to database
                await _dataService.SaveDeviceInfoAsync(_deviceInfo);

                btnSaveDevice.Enabled = true;
                btnSaveDevice.Text = "Save to DB";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully saved device information to the database.",
                    "Device Info Saved", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                btnSaveDevice.Enabled = true;
                btnSaveDevice.Text = "Save to DB";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error saving device information: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void UpdateDeviceInfoDisplay()
        {
            // Clear existing data
            dgvDevice.Rows.Clear();

            // Add device information to the data grid
            foreach (var info in _deviceInfo)
            {
                dgvDevice.Rows.Add(
                    info.Key,
                    info.Value
                );
            }
        }

        #endregion

        #region Accounts Tab

        private async void BtnLoadAccounts_Click(object? sender, EventArgs e)
        {
            try
            {
                if (!_adbTool.IsDeviceConnected())
                {
                    MessageBox.Show("No Android device connected. Please connect a device and try again.",
                        "Device Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                    return;
                }                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnLoadAccount.Enabled = false;
                btnLoadAccount.Text = "Loading...";

                // Load accounts from device
                _accounts = await Task.Run(() => _adbTool.GetAccounts());

                // Update the UI
                UpdateAccountsDisplay();

                btnLoadAccount.Enabled = true;
                btnLoadAccount.Text = "Load from Device";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully loaded {_accounts.Count} accounts from the device.",
                    "Accounts Loaded", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {                btnLoadAccount.Enabled = true;
                btnLoadAccount.Text = "Load from Device";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error loading accounts: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async void BtnSaveAccounts_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_accounts.Count == 0)
                {
                    MessageBox.Show("No accounts to save. Please load accounts first.",
                        "No Data", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }                // Show progress indicator
                Cursor = Cursors.WaitCursor;
                btnSaveAccount.Enabled = false;
                btnSaveAccount.Text = "Saving...";

                // Save accounts to database
                await _dataService.SaveAccountsAsync(_accounts);

                btnSaveAccount.Enabled = true;
                btnSaveAccount.Text = "Save to DB";
                Cursor = Cursors.Default;

                MessageBox.Show($"Successfully saved {_accounts.Count} accounts to the database.",
                    "Accounts Saved", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {                btnSaveAccount.Enabled = true;
                btnSaveAccount.Text = "Save to DB";
                Cursor = Cursors.Default;
                MessageBox.Show($"Error saving accounts: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void UpdateAccountsDisplay()
        {
            // Clear existing data
            dgvAccounts.Rows.Clear();

            // Add accounts to the data grid
            foreach (var account in _accounts)
            {
                dgvAccounts.Rows.Add(
                    account.AccountType,
                    account.Email
                );
            }
        }

        #endregion
    }
}
