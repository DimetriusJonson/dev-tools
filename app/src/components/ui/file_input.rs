use leptos::{html, prelude::*};

#[component]
pub fn FileInput(
    node_ref: NodeRef<html::Input>
) -> impl IntoView {
    view! {
        <input node_ref=node_ref type="file" class="
            cursor-pointer file:cursor-pointer
            block 
            w-full 
            font-medium 
            file:h-7 md:file:h-10 
            text-sm md:text-base 
            text-white
            file:mr-4 
            md:file:py-2 
            file:px-4
            file:rounded-md 
            file:border-0
            file:text-sm md:file:text-base 
            file:bg-gray-200 file:text-black
            hover:file:bg-gray-300 
            dark:hover:file:bg-gray-50" 
            />
    }
}
