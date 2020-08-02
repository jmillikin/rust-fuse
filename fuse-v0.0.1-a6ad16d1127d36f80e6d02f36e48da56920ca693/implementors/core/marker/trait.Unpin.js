(function() {var implementors = {};
implementors["fuse"] = [{"text":"impl&lt;Handlers, Mount&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/struct.FuseServer.html\" title=\"struct fuse::FuseServer\">FuseServer</a>&lt;Handlers, Mount&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Mount: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["fuse::fuse_server::FuseServer"]},{"text":"impl&lt;Handlers, MountOptions&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/struct.FuseServerBuilder.html\" title=\"struct fuse::FuseServerBuilder\">FuseServerBuilder</a>&lt;Handlers, MountOptions&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Handlers: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;MountOptions: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["fuse::fuse_server::FuseServerBuilder"]},{"text":"impl&lt;Handlers&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/struct.FuseServerExecutor.html\" title=\"struct fuse::FuseServerExecutor\">FuseServerExecutor</a>&lt;Handlers&gt;","synthetic":true,"types":["fuse::fuse_server::FuseServerExecutor"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/struct.ServerContext.html\" title=\"struct fuse::ServerContext\">ServerContext</a>","synthetic":true,"types":["fuse::server::ServerContext"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/struct.ProtocolVersion.html\" title=\"struct fuse::ProtocolVersion\">ProtocolVersion</a>","synthetic":true,"types":["fuse::internal::types::ProtocolVersion"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/os/linux/struct.FuseMountOptions.html\" title=\"struct fuse::os::linux::FuseMountOptions\">FuseMountOptions</a>","synthetic":true,"types":["fuse::os::linux::linux_mount_options::FuseMountOptions"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/os/linux/struct.FuseMount.html\" title=\"struct fuse::os::linux::FuseMount\">FuseMount</a>","synthetic":true,"types":["fuse::os::linux::linux_mount_options::FuseMount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.AccessRequest.html\" title=\"struct fuse::protocol::AccessRequest\">AccessRequest</a>","synthetic":true,"types":["fuse::protocol::access::AccessRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.AccessResponse.html\" title=\"struct fuse::protocol::AccessResponse\">AccessResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::access::AccessResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.BmapRequest.html\" title=\"struct fuse::protocol::BmapRequest\">BmapRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::bmap::BmapRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.BmapResponse.html\" title=\"struct fuse::protocol::BmapResponse\">BmapResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::bmap::BmapResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.CreateRequest.html\" title=\"struct fuse::protocol::CreateRequest\">CreateRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::create::CreateRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.CreateResponse.html\" title=\"struct fuse::protocol::CreateResponse\">CreateResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::create::CreateResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FallocateRequest.html\" title=\"struct fuse::protocol::FallocateRequest\">FallocateRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fallocate::FallocateRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FallocateResponse.html\" title=\"struct fuse::protocol::FallocateResponse\">FallocateResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fallocate::FallocateResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FlushRequest.html\" title=\"struct fuse::protocol::FlushRequest\">FlushRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::flush::FlushRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FlushResponse.html\" title=\"struct fuse::protocol::FlushResponse\">FlushResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::flush::FlushResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ForgetRequest.html\" title=\"struct fuse::protocol::ForgetRequest\">ForgetRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::forget::ForgetRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ForgetNode.html\" title=\"struct fuse::protocol::ForgetNode\">ForgetNode</a>","synthetic":true,"types":["fuse::protocol::forget::ForgetNode"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FsyncRequest.html\" title=\"struct fuse::protocol::FsyncRequest\">FsyncRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fsync::FsyncRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FsyncResponse.html\" title=\"struct fuse::protocol::FsyncResponse\">FsyncResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fsync::FsyncResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FsyncdirRequest.html\" title=\"struct fuse::protocol::FsyncdirRequest\">FsyncdirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fsyncdir::FsyncdirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FsyncdirResponse.html\" title=\"struct fuse::protocol::FsyncdirResponse\">FsyncdirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::fsyncdir::FsyncdirResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FuseInitRequest.html\" title=\"struct fuse::protocol::FuseInitRequest\">FuseInitRequest</a>","synthetic":true,"types":["fuse::protocol::fuse_init::FuseInitRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FuseInitResponse.html\" title=\"struct fuse::protocol::FuseInitResponse\">FuseInitResponse</a>","synthetic":true,"types":["fuse::protocol::fuse_init::FuseInitResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FuseInitFlag.html\" title=\"struct fuse::protocol::FuseInitFlag\">FuseInitFlag</a>","synthetic":true,"types":["fuse::protocol::fuse_init::FuseInitFlag"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.FuseInitFlags.html\" title=\"struct fuse::protocol::FuseInitFlags\">FuseInitFlags</a>","synthetic":true,"types":["fuse::protocol::fuse_init::FuseInitFlags"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetattrRequest.html\" title=\"struct fuse::protocol::GetattrRequest\">GetattrRequest</a>","synthetic":true,"types":["fuse::protocol::getattr::GetattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetattrResponse.html\" title=\"struct fuse::protocol::GetattrResponse\">GetattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::getattr::GetattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetlkRequest.html\" title=\"struct fuse::protocol::GetlkRequest\">GetlkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::getlk::GetlkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetlkResponse.html\" title=\"struct fuse::protocol::GetlkResponse\">GetlkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::getlk::GetlkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetxattrRequest.html\" title=\"struct fuse::protocol::GetxattrRequest\">GetxattrRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::getxattr::GetxattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.GetxattrResponse.html\" title=\"struct fuse::protocol::GetxattrResponse\">GetxattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::getxattr::GetxattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.IoctlRequest.html\" title=\"struct fuse::protocol::IoctlRequest\">IoctlRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::ioctl::IoctlRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.IoctlResponse.html\" title=\"struct fuse::protocol::IoctlResponse\">IoctlResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::ioctl::IoctlResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LinkRequest.html\" title=\"struct fuse::protocol::LinkRequest\">LinkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::link::LinkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LinkResponse.html\" title=\"struct fuse::protocol::LinkResponse\">LinkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::link::LinkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ListxattrRequest.html\" title=\"struct fuse::protocol::ListxattrRequest\">ListxattrRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::listxattr::ListxattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ListxattrResponse.html\" title=\"struct fuse::protocol::ListxattrResponse\">ListxattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::listxattr::ListxattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LookupRequest.html\" title=\"struct fuse::protocol::LookupRequest\">LookupRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::lookup::LookupRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LookupResponse.html\" title=\"struct fuse::protocol::LookupResponse\">LookupResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::lookup::LookupResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LseekRequest.html\" title=\"struct fuse::protocol::LseekRequest\">LseekRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::lseek::LseekRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.LseekResponse.html\" title=\"struct fuse::protocol::LseekResponse\">LseekResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::lseek::LseekResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.MkdirRequest.html\" title=\"struct fuse::protocol::MkdirRequest\">MkdirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::mkdir::MkdirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.MkdirResponse.html\" title=\"struct fuse::protocol::MkdirResponse\">MkdirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::mkdir::MkdirResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.MknodRequest.html\" title=\"struct fuse::protocol::MknodRequest\">MknodRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::mknod::MknodRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.MknodResponse.html\" title=\"struct fuse::protocol::MknodResponse\">MknodResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::mknod::MknodResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.OpenRequest.html\" title=\"struct fuse::protocol::OpenRequest\">OpenRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::open::OpenRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.OpenResponse.html\" title=\"struct fuse::protocol::OpenResponse\">OpenResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::open::OpenResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.OpendirRequest.html\" title=\"struct fuse::protocol::OpendirRequest\">OpendirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::opendir::OpendirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.OpendirResponse.html\" title=\"struct fuse::protocol::OpendirResponse\">OpendirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::opendir::OpendirResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReadRequest.html\" title=\"struct fuse::protocol::ReadRequest\">ReadRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::read::ReadRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReadResponse.html\" title=\"struct fuse::protocol::ReadResponse\">ReadResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::read::ReadResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReaddirRequest.html\" title=\"struct fuse::protocol::ReaddirRequest\">ReaddirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::readdir::ReaddirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReaddirResponse.html\" title=\"struct fuse::protocol::ReaddirResponse\">ReaddirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::readdir::ReaddirResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.Dirent.html\" title=\"struct fuse::protocol::Dirent\">Dirent</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::readdir::Dirent"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReadlinkRequest.html\" title=\"struct fuse::protocol::ReadlinkRequest\">ReadlinkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::readlink::ReadlinkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReadlinkResponse.html\" title=\"struct fuse::protocol::ReadlinkResponse\">ReadlinkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::readlink::ReadlinkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReleaseRequest.html\" title=\"struct fuse::protocol::ReleaseRequest\">ReleaseRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::release::ReleaseRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReleaseResponse.html\" title=\"struct fuse::protocol::ReleaseResponse\">ReleaseResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::release::ReleaseResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReleasedirRequest.html\" title=\"struct fuse::protocol::ReleasedirRequest\">ReleasedirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::releasedir::ReleasedirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.ReleasedirResponse.html\" title=\"struct fuse::protocol::ReleasedirResponse\">ReleasedirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::releasedir::ReleasedirResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RemovexattrRequest.html\" title=\"struct fuse::protocol::RemovexattrRequest\">RemovexattrRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::removexattr::RemovexattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RemovexattrResponse.html\" title=\"struct fuse::protocol::RemovexattrResponse\">RemovexattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::removexattr::RemovexattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RenameRequest.html\" title=\"struct fuse::protocol::RenameRequest\">RenameRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::rename::RenameRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RenameResponse.html\" title=\"struct fuse::protocol::RenameResponse\">RenameResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::rename::RenameResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RmdirRequest.html\" title=\"struct fuse::protocol::RmdirRequest\">RmdirRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::rmdir::RmdirRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.RmdirResponse.html\" title=\"struct fuse::protocol::RmdirResponse\">RmdirResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::rmdir::RmdirResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetattrRequest.html\" title=\"struct fuse::protocol::SetattrRequest\">SetattrRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setattr::SetattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetattrResponse.html\" title=\"struct fuse::protocol::SetattrResponse\">SetattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setattr::SetattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetlkRequest.html\" title=\"struct fuse::protocol::SetlkRequest\">SetlkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setlk::SetlkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetlkResponse.html\" title=\"struct fuse::protocol::SetlkResponse\">SetlkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setlk::SetlkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetxattrRequest.html\" title=\"struct fuse::protocol::SetxattrRequest\">SetxattrRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setxattr::SetxattrRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SetxattrResponse.html\" title=\"struct fuse::protocol::SetxattrResponse\">SetxattrResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::setxattr::SetxattrResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.StatfsRequest.html\" title=\"struct fuse::protocol::StatfsRequest\">StatfsRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::statfs::StatfsRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.StatfsResponse.html\" title=\"struct fuse::protocol::StatfsResponse\">StatfsResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::statfs::StatfsResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SymlinkRequest.html\" title=\"struct fuse::protocol::SymlinkRequest\">SymlinkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::symlink::SymlinkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.SymlinkResponse.html\" title=\"struct fuse::protocol::SymlinkResponse\">SymlinkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::symlink::SymlinkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.UnlinkRequest.html\" title=\"struct fuse::protocol::UnlinkRequest\">UnlinkRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::unlink::UnlinkRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.UnlinkResponse.html\" title=\"struct fuse::protocol::UnlinkResponse\">UnlinkResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::unlink::UnlinkResponse"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.WriteRequest.html\" title=\"struct fuse::protocol::WriteRequest\">WriteRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::write::WriteRequest"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.WriteResponse.html\" title=\"struct fuse::protocol::WriteResponse\">WriteResponse</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::write::WriteResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.Node.html\" title=\"struct fuse::protocol::Node\">Node</a>","synthetic":true,"types":["fuse::protocol::node::Node"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.NodeAttr.html\" title=\"struct fuse::protocol::NodeAttr\">NodeAttr</a>","synthetic":true,"types":["fuse::protocol::node::NodeAttr"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.NodeId.html\" title=\"struct fuse::protocol::NodeId\">NodeId</a>","synthetic":true,"types":["fuse::protocol::node::NodeId"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.NodeKind.html\" title=\"struct fuse::protocol::NodeKind\">NodeKind</a>","synthetic":true,"types":["fuse::protocol::node::NodeKind"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"fuse/protocol/struct.UnknownRequest.html\" title=\"struct fuse::protocol::UnknownRequest\">UnknownRequest</a>&lt;'a&gt;","synthetic":true,"types":["fuse::protocol::unknown::UnknownRequest"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()