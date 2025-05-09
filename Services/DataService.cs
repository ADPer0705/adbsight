using android_debugger.Data;
using android_debugger.Models;
using Microsoft.EntityFrameworkCore;

namespace android_debugger.Services
{
    public class DataService
    {
        private readonly AndroidDbContext _dbContext;

        public DataService(AndroidDbContext dbContext)
        {
            _dbContext = dbContext;
        }

        // CONTACTS
        public async Task SaveContactsAsync(List<Contact> contacts)
        {
            if (contacts == null || !contacts.Any())
                return;

            // Get existing phone numbers to avoid duplicates
            var existingPhoneNumbers = await _dbContext.Contacts
                .Select(c => c.PhoneNumber)
                .ToListAsync();

            // Filter out contacts that already exist in the database
            var newContacts = contacts
                .Where(c => !string.IsNullOrEmpty(c.PhoneNumber) && !existingPhoneNumbers.Contains(c.PhoneNumber))
                .ToList();

            if (newContacts.Any())
            {
                await _dbContext.Contacts.AddRangeAsync(newContacts);
                await _dbContext.SaveChangesAsync();
            }
        }

        public async Task<List<Contact>> GetContactsAsync()
        {
            return await _dbContext.Contacts.OrderBy(c => c.Name).ToListAsync();
        }

        // SMS
        public async Task SaveSMSAsync(List<SMS> messages)
        {
            if (messages == null || !messages.Any())
                return;

            // For SMS we can use the combination of address, body and date to check for duplicates
            foreach (var message in messages)
            {
                var exists = await _dbContext.SMSMessages
                    .AnyAsync(s => s.Address == message.Address && 
                                  s.Body == message.Body && 
                                  s.Date == message.Date);

                if (!exists)
                {
                    _dbContext.SMSMessages.Add(message);
                }
            }

            await _dbContext.SaveChangesAsync();
        }

        public async Task<List<SMS>> GetSMSAsync()
        {
            return await _dbContext.SMSMessages.OrderByDescending(s => s.Date).ToListAsync();
        }

        // CALL LOGS
        public async Task SaveCallLogsAsync(List<CallLog> calls)
        {
            if (calls == null || !calls.Any())
                return;

            // For call logs we can use the combination of number, date and duration to check for duplicates
            foreach (var call in calls)
            {
                var exists = await _dbContext.CallLogs
                    .AnyAsync(c => c.Number == call.Number && 
                                  c.Date == call.Date && 
                                  c.Duration == call.Duration);

                if (!exists)
                {
                    _dbContext.CallLogs.Add(call);
                }
            }

            await _dbContext.SaveChangesAsync();
        }

        public async Task<List<CallLog>> GetCallLogsAsync()
        {
            return await _dbContext.CallLogs.OrderByDescending(c => c.Date).ToListAsync();
        }

        // DEVICE INFO
        public async Task SaveDeviceInfoAsync(List<DeviceInfo> deviceInfo)
        {
            if (deviceInfo == null || !deviceInfo.Any())
                return;

            // For device info, we'll update existing entries if the key already exists
            foreach (var info in deviceInfo)
            {
                var existingInfo = await _dbContext.DeviceInfo
                    .FirstOrDefaultAsync(d => d.Key == info.Key);

                if (existingInfo == null)
                {
                    _dbContext.DeviceInfo.Add(info);
                }
                else
                {
                    existingInfo.Value = info.Value;
                    existingInfo.CreatedAt = DateTime.Now;
                }
            }

            await _dbContext.SaveChangesAsync();
        }

        public async Task<List<DeviceInfo>> GetDeviceInfoAsync()
        {
            return await _dbContext.DeviceInfo.OrderBy(d => d.Key).ToListAsync();
        }

        // ACCOUNTS
        public async Task SaveAccountsAsync(List<Account> accounts)
        {
            if (accounts == null || !accounts.Any())
                return;

            // For accounts, we can use the combination of account type and email to check for duplicates
            foreach (var account in accounts)
            {
                var exists = await _dbContext.Accounts
                    .AnyAsync(a => a.AccountType == account.AccountType && 
                                  a.Email == account.Email);

                if (!exists)
                {
                    _dbContext.Accounts.Add(account);
                }
            }

            await _dbContext.SaveChangesAsync();
        }

        public async Task<List<Account>> GetAccountsAsync()
        {
            return await _dbContext.Accounts.OrderBy(a => a.AccountType).ToListAsync();
        }
    }
}
