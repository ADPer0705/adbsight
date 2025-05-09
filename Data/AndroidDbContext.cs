using Microsoft.EntityFrameworkCore;
using android_debugger.Models;

namespace android_debugger.Data
{
    public class AndroidDbContext : DbContext
    {
        public AndroidDbContext(DbContextOptions<AndroidDbContext> options) : base(options)
        {
        }

        public DbSet<Contact> Contacts { get; set; }
        public DbSet<SMS> SMSMessages { get; set; }
        public DbSet<CallLog> CallLogs { get; set; }
        public DbSet<DeviceInfo> DeviceInfo { get; set; }
        public DbSet<Account> Accounts { get; set; }

        protected override void OnModelCreating(ModelBuilder modelBuilder)
        {
            base.OnModelCreating(modelBuilder);
            
            // Additional configuration if needed
            modelBuilder.Entity<Contact>()
                .HasIndex(c => c.PhoneNumber);

            modelBuilder.Entity<SMS>()
                .HasIndex(s => s.Address);

            modelBuilder.Entity<CallLog>()
                .HasIndex(c => c.Number);
        }
    }
}
