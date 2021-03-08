#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
		
		/// 证明的存储项目
        	/// 它将证明映射到提出声明的用户以及声明的时间。
        	Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		
		/// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// 该证明已经被声明
		ProofAlreadyClaimed,
		/// 该证明不存在，因此它不能被撤销
		NoSuchProof,
		/// 该证明已经被另一个账号声明，因此它不能被撤销
		NotProofOwner,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}
		
		/// 允许用户队未声明的证明拥有所有权
		#[weight = 10_000]
		fn create_claim(origin, proof: Vec<u8>) {
		    // 检查 extrinsic 是否签名并获得签名者
		    // 如果 extrinsic 未签名，此函数将返回一个错误。
		    // https://substrate.dev/docs/en/knowledgebase/runtime/origin
		    let sender = ensure_signed(origin)?;

		    // 校验指定的证明是否被声明
		    ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

		    // 从 FRAME 系统模块中获取区块号.
		    let current_block = <frame_system::Module<T>>::block_number();

		    // 存储证明：发送人与区块号
		    Proofs::<T>::insert(&proof, (&sender, current_block));

		    // 声明创建后，发送事件
		    Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
		}

		/// 允许证明所有者撤回声明
		#[weight = 10_000]
		fn revoke_claim(origin, proof: Vec<u8>) {
		    //  检查 extrinsic 是否签名并获得签名者
		    // 如果 extrinsic 未签名，此函数将返回一个错误。
		    // https://substrate.dev/docs/en/knowledgebase/runtime/origin
		    let sender = ensure_signed(origin)?;

		    // 校验指定的证明是否被声明
		    ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

		    // 获取声明的所有者
		    let (owner, _) = Proofs::<T>::get(&proof);

		    // 验证当前的调用者是证声明的所有者
		    ensure!(sender == owner, Error::<T>::NotProofOwner);

		    // 从存储中移除声明
		    Proofs::<T>::remove(&proof);

		    // 声明抹掉后，发送事件
		    Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
		}
	}
}
