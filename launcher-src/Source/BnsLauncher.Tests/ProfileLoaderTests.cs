using System.IO.Abstractions.TestingHelpers;
using System.Linq;
using System.Threading.Tasks;
using BnsLauncher.Core.Abstractions;
using BnsLauncher.Core.Services;
using BnsLauncher.Tests.Helpers;
using BnsLauncher.Tests.Mocks;
using NUnit.Framework;

namespace BnsLauncher.Tests
{
    [TestFixture]
    public class ProfileLoaderTests
    {
        private ILogger _logger;

        [SetUp]
        public void SetUp()
        {
            _logger = new MockLogger();
        }

        [Test]
        public async Task SamplePrivateServerProfile()
        {
            var fs = new MockFileSystem();
            fs.WriteSampleProfile(
                "C:\\Users\\TestUser\\AppData\\Local\\BnSLauncher\\Profiles\\PrivateServer",
                "PrivateServer");

            var profileLoader = new ProfileLoader(fs, _logger, new NullSampleProfileWriter());
            var profiles = await profileLoader.LoadProfiles(
                "C:\\Users\\TestUser\\AppData\\Local\\BnSLauncher\\Profiles");

            Assert.AreEqual(1, profiles.Count, "Must only load 1 profile");

            var profile = profiles.First();

            Assert.AreEqual(
                "C:\\Users\\TestUser\\AppData\\Local\\BnSLauncher\\Profiles\\PrivateServer",
                profile.ProfilePath,
                "Profile paths must be equal");

            Assert.IsNull(profile.BnsPatchPath, "Private server profile has no bnspatch.xml");

            Assert.AreEqual("Private Server", profile.Name, "Names must be equal");
            Assert.AreEqual('P', profile.Initial, "Initials must be equal");
            Assert.AreEqual("red", profile.Background, "Backgrounds must be equal");
            Assert.AreEqual("white", profile.Foreground, "Foregrounds must be equal");

            Assert.AreEqual("/LaunchByLauncher /LoginMode 2", profile.Arguments, "Arguments must select the password login path");

            Assert.IsNull(profile.BinPath, "Bin path must be null");
            Assert.IsNull(profile.LocalBinPath, "Local bin path must be null");

            Assert.IsFalse(profile.AllowAccounts, "Login happens in the client panel");
        }

        // TODO: Test profile that tests untested stuff from sample profiles
        // AutopinOnRelog
    }
}