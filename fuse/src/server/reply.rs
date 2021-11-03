// Copyright 2021 John Millikin and the rust-fuse contributors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use crate::io::{AsyncOutputStream, OutputStream, SendError};

pub trait Reply {
	fn send<Stream: OutputStream>(
		&self,
		stream: &Stream,
		reply_info: ReplyInfo,
	) -> Result<(), SendError<Stream::Error>>;

	fn send_async<Stream: AsyncOutputStream>(
		&self,
		stream: &Stream,
		reply_info: ReplyInfo,
	) -> Stream::Future;
}

pub struct ReplyInfo {
	pub(super) request_id: u64,
	pub(super) version_minor: u32,
}

mod impls {
	use crate::io::encode::{self, EncodeReply};
	use crate::io::{AsyncOutputStream, OutputStream, SendError};
	use crate::protocol::*;

	use super::{Reply, ReplyInfo};

	macro_rules! impl_reply {
		($t:ident) => {
			impl Reply for $t<'_> {
				fn send<Stream: OutputStream>(
					&self,
					stream: &Stream,
					reply_info: ReplyInfo,
				) -> Result<(), SendError<Stream::Error>> {
					self.encode(
						encode::SyncSendOnce::new(stream),
						reply_info.request_id,
						reply_info.version_minor,
					)
				}

				fn send_async<Stream: AsyncOutputStream>(
					&self,
					stream: &Stream,
					reply_info: ReplyInfo,
				) -> Stream::Future {
					self.encode(
						encode::AsyncSendOnce::new(stream),
						reply_info.request_id,
						reply_info.version_minor,
					)
				}
			}
		};
	}

	impl_reply! { AccessResponse      }
	impl_reply! { CreateResponse      }
	impl_reply! { FallocateResponse   }
	impl_reply! { FlushResponse       }
	impl_reply! { FsyncResponse       }
	impl_reply! { FsyncdirResponse    }
	impl_reply! { GetattrResponse     }
	impl_reply! { GetlkResponse       }
	impl_reply! { GetxattrResponse    }
	impl_reply! { LinkResponse        }
	impl_reply! { ListxattrResponse   }
	impl_reply! { LookupResponse      }
	impl_reply! { LseekResponse       }
	impl_reply! { MkdirResponse       }
	impl_reply! { MknodResponse       }
	impl_reply! { OpenResponse        }
	impl_reply! { OpendirResponse     }
	impl_reply! { ReadResponse        }
	impl_reply! { ReaddirResponse     }
	impl_reply! { ReadlinkResponse    }
	impl_reply! { ReleaseResponse     }
	impl_reply! { ReleasedirResponse  }
	impl_reply! { RemovexattrResponse }
	impl_reply! { RenameResponse      }
	impl_reply! { RmdirResponse       }
	impl_reply! { SetlkResponse       }
	impl_reply! { SetxattrResponse    }
	impl_reply! { StatfsResponse      }
	impl_reply! { SymlinkResponse     }
	impl_reply! { UnlinkResponse      }
	impl_reply! { WriteResponse       }

	#[cfg(any(doc, feature = "unstable_bmap"))]
	impl_reply! { BmapResponse }

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	impl_reply! { IoctlResponse }

	#[cfg(any(doc, feature = "unstable_setattr"))]
	impl_reply! { SetattrResponse }
}
